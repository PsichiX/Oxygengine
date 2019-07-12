use crate::client::NativeClient;
use network::{
    client::{Client, ClientID, ClientState, MessageID},
    server::{Server, ServerID, ServerState},
};
use std::{
    collections::{HashMap, VecDeque},
    io::ErrorKind,
    mem::replace,
    net::TcpListener,
    ops::Range,
    sync::{Arc, Mutex},
    thread::{sleep, Builder as ThreadBuilder, JoinHandle},
    time::Duration,
};

const LISTENER_SLEEP_MS: u64 = 10;

type MsgData = (ClientID, MessageID, Vec<u8>);

pub struct NativeServer {
    id: ServerID,
    state: Arc<Mutex<ServerState>>,
    clients: Arc<Mutex<HashMap<ClientID, NativeClient>>>,
    clients_ids_cached: Vec<ClientID>,
    messages: VecDeque<MsgData>,
    thread: Option<JoinHandle<()>>,
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
}

impl Server for NativeServer {
    fn open(url: &str) -> Option<Self> {
        let sid = ServerID::default();
        let url = url.to_owned();
        let state = Arc::new(Mutex::new(ServerState::Starting));
        let state2 = state.clone();
        let clients = Arc::new(Mutex::new(HashMap::default()));
        let clients2 = clients.clone();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Server: {:?}", sid))
                .spawn(move || {
                    let listener = TcpListener::bind(&url).unwrap();
                    listener.set_nonblocking(true).unwrap_or_else(|_| {
                        panic!(
                            "Server {:?} cannot set non-blocking listening on: {}",
                            sid, &url
                        )
                    });
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
                                let client = NativeClient::from(stream);
                                let id = client.id();
                                clients2.lock().unwrap().insert(id, client);
                            }
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                sleep(Duration::from_millis(LISTENER_SLEEP_MS));
                            }
                            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                break;
                            }
                            Err(e) => {
                                panic!("Server {:?} listener {} got IO error: {}", sid, &url, e)
                            }
                        }
                    }
                    {
                        *state2.lock().unwrap() = ServerState::Closed;
                    }
                })
                .unwrap(),
        );
        Some(Self {
            id: sid,
            state,
            clients,
            clients_ids_cached: vec![],
            messages: Default::default(),
            thread,
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
        if let Some(client) = clients.remove(&id) {
            client.close();
        }
    }

    fn disconnect_all(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        for (_, client) in clients.drain() {
            client.close();
        }
    }

    fn send(&mut self, id: ClientID, msg_id: MessageID, data: &[u8]) -> Option<Range<usize>> {
        if self.state() != ServerState::Open {
            return None;
        }
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(&id) {
            if let Some(size) = client.send(msg_id, data) {
                return Some(size);
            }
        }
        None
    }

    fn send_all(&mut self, id: MessageID, data: &[u8]) {
        if self.state() != ServerState::Open {
            return;
        }
        let mut clients = self.clients.lock().unwrap();
        for client in clients.values_mut() {
            drop(client.send(id, data));
        }
    }

    fn read(&mut self) -> Option<(ClientID, MessageID, Vec<u8>)> {
        self.messages.pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        self.messages.drain(..).collect()
    }

    fn process(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        for (id, client) in clients.iter_mut() {
            self.messages.extend(
                client
                    .read_all()
                    .into_iter()
                    .map(|(mid, data)| (*id, mid, data)),
            );
        }
        clients.retain(|_, client| client.state() != ClientState::Closed);
        self.clients_ids_cached.clear();
        for id in clients.keys() {
            self.clients_ids_cached.push(*id);
        }
    }
}
