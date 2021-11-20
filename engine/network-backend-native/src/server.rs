use crate::{client::NativeClient, utils::DoOnDrop};
use network::{
    client::{Client, ClientId, ClientState, MessageId},
    server::{Server, ServerId, ServerState},
};
use std::{
    collections::{HashMap, VecDeque},
    io::ErrorKind,
    mem::replace,
    net::TcpListener,
    ops::Range,
    sync::{Arc, RwLock},
    thread::{sleep, Builder as ThreadBuilder, JoinHandle},
    time::Duration,
};

const LISTENER_SLEEP_MS: u64 = 10;

type MsgData = (ClientId, MessageId, Vec<u8>);

pub struct NativeServer {
    id: ServerId,
    state: Arc<RwLock<ServerState>>,
    clients: Arc<RwLock<HashMap<ClientId, NativeClient>>>,
    clients_ids_cached: Vec<ClientId>,
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
        if let Ok(mut state) = self.state.write() {
            *state = ServerState::Closed;
        }
        let thread = replace(&mut self.thread, None);
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }
}

impl Server for NativeServer {
    fn open(url: &str) -> Option<Self> {
        let sid = ServerId::default();
        let url = url.to_owned();
        let state = Arc::new(RwLock::new(ServerState::Starting));
        let state2 = state.clone();
        let clients = Arc::new(RwLock::new(HashMap::default()));
        let clients2 = clients.clone();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Server: {:?}", sid))
                .spawn(move || {
                    let state3 = state2.clone();
                    let _ = DoOnDrop::new(move || {
                        if let Ok(mut state) = state3.write() {
                            *state = ServerState::Closed;
                        }
                    });
                    let listener = TcpListener::bind(&url).unwrap();
                    listener.set_nonblocking(true).unwrap_or_else(|_| {
                        panic!(
                            "Server {:?} cannot set non-blocking listening on: {}",
                            sid, &url
                        )
                    });
                    if let Ok(mut state) = state2.write() {
                        *state = ServerState::Open;
                    }
                    for stream in listener.incoming() {
                        if let Ok(state) = state2.read() {
                            if *state == ServerState::Closed {
                                break;
                            }
                        }
                        match stream {
                            Ok(stream) => {
                                let client = NativeClient::from(stream);
                                let id = client.id();
                                if let Ok(mut clients) = clients2.write() {
                                    clients.insert(id, client);
                                }
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
                    if let Ok(mut state) = state2.write() {
                        *state = ServerState::Closed;
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

    fn id(&self) -> ServerId {
        self.id
    }

    fn state(&self) -> ServerState {
        if let Ok(state) = self.state.read() {
            *state
        } else {
            ServerState::default()
        }
    }

    fn clients(&self) -> &[ClientId] {
        &self.clients_ids_cached
    }

    fn disconnect(&mut self, id: ClientId) {
        if let Ok(mut clients) = self.clients.write() {
            if let Some(client) = clients.remove(&id) {
                client.close();
            }
        }
    }

    fn disconnect_all(&mut self) {
        if let Ok(mut clients) = self.clients.write() {
            for (_, client) in clients.drain() {
                client.close();
            }
        }
    }

    fn send(&mut self, id: ClientId, msg_id: MessageId, data: &[u8]) -> Option<Range<usize>> {
        if self.state() != ServerState::Open {
            return None;
        }
        if let Ok(mut clients) = self.clients.write() {
            if let Some(client) = clients.get_mut(&id) {
                if let Some(size) = client.send(msg_id, data) {
                    return Some(size);
                }
            }
        }
        None
    }

    fn send_all(&mut self, id: MessageId, data: &[u8]) {
        if self.state() != ServerState::Open {
            return;
        }
        if let Ok(mut clients) = self.clients.write() {
            for client in clients.values_mut() {
                drop(client.send(id, data));
            }
        }
    }

    fn read(&mut self) -> Option<(ClientId, MessageId, Vec<u8>)> {
        self.messages.pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        self.messages.drain(..).collect()
    }

    fn process(&mut self) {
        if let Ok(mut clients) = self.clients.write() {
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
}
