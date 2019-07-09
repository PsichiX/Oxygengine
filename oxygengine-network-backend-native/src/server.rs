use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use network::{
    client::{ClientID, ClientState, MessageID},
    server::{Server, ServerID, ServerState},
};
use std::{
    collections::{HashMap, VecDeque},
    io::{Cursor, ErrorKind, Read, Write},
    mem::replace,
    net::{Shutdown, TcpListener, TcpStream},
    ops::Range,
    sync::{Arc, Mutex},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

const LISTENER_SLEEP_MS: u64 = 10;

type MsgData = (ClientID, MessageID, Vec<u8>);

struct Client {
    id: ClientID,
    stream: Option<TcpStream>,
    state: ClientState,
}

pub struct NativeServer {
    id: ServerID,
    state: Arc<Mutex<ServerState>>,
    clients: Arc<Mutex<HashMap<ClientID, Arc<Mutex<Client>>>>>,
    clients_ids_cached: Vec<ClientID>,
    messages: VecDeque<MsgData>,
    thread: Option<JoinHandle<()>>,
    receive_buffer: Vec<u8>,
}

impl Drop for NativeServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl NativeServer {
    fn cleanup(&mut self) {
        {
            *self.state.lock().unwrap() = ServerState::Closed;
        }
        let thread = replace(&mut self.thread, None);
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }

    fn read_message(client_id: ClientID, buffer: &[u8], size: usize) -> Option<MsgData> {
        if size >= 8 {
            let mut stream = Cursor::new(buffer);
            let id = stream.read_u32::<BigEndian>().unwrap();
            let version = stream.read_u32::<BigEndian>().unwrap();
            let mut data = Vec::with_capacity(size - 8);
            stream.read_to_end(&mut data).unwrap();
            Some((client_id, MessageID::new(id, version), data))
        } else {
            None
        }
    }
}

impl Server for NativeServer {
    fn open(url: &str) -> Option<Self> {
        let sid = ServerID::default();
        let url = url.to_owned();
        let state = Arc::new(Mutex::new(ServerState::Starting));
        let state2 = state.clone();
        let clients = Arc::new(Mutex::new(HashMap::default()));
        let clients2 = clients.clone();
        let thread = Some(spawn(move || {
            let listener = TcpListener::bind(&url).unwrap();
            listener.set_nonblocking(true).expect(&format!(
                "Server {:?} cannot set non-blocking listening on: {}",
                sid, &url
            ));
            {
                *state2.lock().unwrap() = ServerState::Open;
            }
            for stream in listener.incoming() {
                {
                    if *state2.lock().unwrap() == ServerState::Closed {
                        break;
                    }
                }
                match stream {
                    Ok(stream) => {
                        let id = ClientID::default();
                        stream.set_nonblocking(true).expect(&format!(
                            "Server {:?} client {:?} cannot set non-blocking streaming on: {}",
                            sid, id, &url
                        ));
                        stream.set_nodelay(true).expect(&format!(
                            "Server {:?} client {:?} cannot set no-delay streaming on: {}",
                            sid, id, &url,
                        ));
                        let client = Arc::new(Mutex::new(Client {
                            id,
                            stream: Some(stream),
                            state: ClientState::Open,
                        }));
                        clients2.lock().unwrap().insert(id, client);
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        sleep(Duration::from_millis(LISTENER_SLEEP_MS));
                        continue;
                    }
                    Err(e) => panic!("Server {:?} listener {} got IO error: {}", sid, &url, e),
                }
            }
            {
                *state2.lock().unwrap() = ServerState::Closed;
            }
        }));
        Some(Self {
            id: sid,
            state,
            clients,
            clients_ids_cached: vec![],
            messages: Default::default(),
            thread,
            receive_buffer: vec![0; 1024],
        })
    }

    fn close(mut self) -> Self {
        self.cleanup();
        self
    }

    fn id(&self) -> ServerID {
        self.id
    }

    fn state(&self) -> ServerState {
        *self.state.lock().unwrap()
    }

    fn clients(&self) -> &[ClientID] {
        &self.clients_ids_cached
    }

    fn disconnect(&mut self, id: ClientID) {
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(&id) {
            let mut client = client.lock().unwrap();
            client.state = ClientState::Closed;
            if let Some(stream) = &client.stream {
                stream.shutdown(Shutdown::Both).unwrap();
            }
            client.stream = None;
        }
        clients.remove(&id);
    }

    fn disconnect_all(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        for client in clients.values() {
            let mut client = client.lock().unwrap();
            client.state = ClientState::Closed;
            if let Some(stream) = &client.stream {
                stream.shutdown(Shutdown::Both).unwrap();
            }
            client.stream = None;
        }
        clients.clear();
    }

    fn send(&mut self, id: ClientID, msg_id: MessageID, data: &[u8]) -> Option<Range<usize>> {
        if self.state() != ServerState::Open {
            return None;
        }
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(&id) {
            let size = data.len();
            let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
            drop(stream.write_u32::<BigEndian>(msg_id.id()));
            drop(stream.write_u32::<BigEndian>(msg_id.version()));
            drop(stream.write(data));
            let data = stream.into_inner();
            let mut client = client.lock().unwrap();
            if client.state == ClientState::Open {
                if let Some(cs) = &mut client.stream {
                    if cs.write_all(&data).is_ok() {
                        return Some(0..size);
                    }
                }
            }
        }
        None
    }

    fn send_all(&mut self, id: MessageID, data: &[u8]) {
        if self.state() != ServerState::Open {
            return;
        }
        let size = data.len();
        let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
        drop(stream.write_u32::<BigEndian>(id.id()));
        drop(stream.write_u32::<BigEndian>(id.version()));
        drop(stream.write(data));
        let data = stream.into_inner();
        let clients = self.clients.lock().unwrap();
        for client in clients.values() {
            let mut client = client.lock().unwrap();
            if client.state == ClientState::Open {
                if let Some(cs) = &mut client.stream {
                    cs.write_all(&data).unwrap();
                }
            }
        }
    }

    fn read(&mut self) -> Option<(ClientID, MessageID, Vec<u8>)> {
        self.messages.pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        let result = self.messages.iter().cloned().collect();
        self.messages.clear();
        result
    }

    fn process(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        for client in clients.values() {
            let mut client = client.lock().unwrap();
            let id = client.id;
            if let Some(stream) = &mut client.stream {
                loop {
                    if let Ok(size) = stream.read_to_end(&mut self.receive_buffer) {
                        if let Some(msg) = Self::read_message(id, &self.receive_buffer, size) {
                            self.messages.push_back(msg);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        clients.retain(|_, client| client.lock().unwrap().state != ClientState::Closed);
        self.clients_ids_cached.clear();
        for id in clients.keys() {
            self.clients_ids_cached.push(*id);
        }
    }
}
