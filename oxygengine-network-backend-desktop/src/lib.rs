extern crate byteorder;
extern crate oxygengine_core as core;
extern crate oxygengine_network as network;
extern crate ws;

pub mod prelude {
    pub use crate::*;
}

#[cfg(test)]
mod tests;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use network::{
    client::{ClientId, ClientState, MessageId},
    server::{Server, ServerId, ServerState},
};
use std::{
    collections::{HashMap, VecDeque},
    io::{Cursor, Read, Write},
    mem::replace,
    ops::Range,
    sync::{Arc, RwLock},
    thread::{spawn, JoinHandle},
};
use ws::{CloseCode, Handler, Handshake, Message, Result, Sender as WsSender, WebSocket};

type MsgData = (ClientId, MessageId, Vec<u8>);

#[derive(Clone)]
struct Client {
    id: ClientId,
    ws: WsSender,
    state: ClientState,
    messages: VecDeque<MsgData>,
}

struct ClientHandler {
    client: Arc<RwLock<Client>>,
}

impl Handler for ClientHandler {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        if let Ok(mut client) = self.client.write() {
            client.state = ClientState::Open;
        }
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Binary(msg) = msg {
            let size = msg.len();
            if size >= 8 {
                let mut stream = Cursor::new(msg);
                let id = stream.read_u32::<BigEndian>().unwrap();
                let version = stream.read_u32::<BigEndian>().unwrap();
                let mut data = Vec::with_capacity(size - 8);
                stream.read_to_end(&mut data).unwrap();
                if let Ok(mut client) = self.client.write() {
                    let client_id = client.id;
                    client
                        .messages
                        .push_back((client_id, MessageId::new(id, version), data));
                }
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        if let Ok(mut client) = self.client.write() {
            client.state = ClientState::Closed;
        }
    }
}

pub struct DesktopServer {
    id: ServerId,
    state: Arc<RwLock<ServerState>>,
    clients: Arc<RwLock<HashMap<ClientId, Arc<RwLock<Client>>>>>,
    clients_ids_cached: Vec<ClientId>,
    messages: VecDeque<MsgData>,
    ws: Arc<RwLock<Option<WsSender>>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for DesktopServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl DesktopServer {
    fn cleanup(&mut self) {
        {
            if let Ok(mut state) = self.state.write() {
                *state = ServerState::Closed;
            }
            if let Ok(mut ws) = self.ws.write() {
                if let Some(ws) = ws.as_ref() {
                    ws.shutdown().unwrap();
                }
                *ws = None;
            }
        }
        let thread = replace(&mut self.thread, None);
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }
}

impl Server for DesktopServer {
    fn open(url: &str) -> Option<Self> {
        let url = url.to_owned();
        let state = Arc::new(RwLock::new(ServerState::Starting));
        let state2 = state.clone();
        let clients = Arc::new(RwLock::new(HashMap::default()));
        let clients2 = clients.clone();
        let sender = Arc::new(RwLock::new(None));
        let sender2 = sender.clone();
        let thread = Some(spawn(move || {
            let ws = WebSocket::new(|ws| {
                let id = ClientId::default();
                let client = Arc::new(RwLock::new(Client {
                    id,
                    ws,
                    state: ClientState::Connecting,
                    messages: VecDeque::new(),
                }));
                if let Ok(mut clients) = clients2.write() {
                    clients.insert(id, client.clone());
                }
                ClientHandler { client }
            })
            .unwrap();
            if let Ok(mut sender) = sender2.write() {
                *sender = Some(ws.broadcaster());
            }
            if let Ok(mut state) = state2.write() {
                *state = ServerState::Open;
            }
            ws.listen(&url).unwrap();
            if let Ok(mut sender) = sender2.write() {
                *sender = None;
            }
            if let Ok(mut state) = state2.write() {
                *state = ServerState::Closed;
            }
        }));
        Some(Self {
            id: Default::default(),
            state,
            clients,
            clients_ids_cached: vec![],
            messages: Default::default(),
            ws: sender,
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
            if let Some(client) = clients.get_mut(&id) {
                if let Ok(client) = client.read() {
                    drop(client.ws.close(CloseCode::Normal));
                    drop(client.ws.shutdown());
                }
            }
            clients.remove(&id);
        }
    }

    fn disconnect_all(&mut self) {
        if let Ok(mut clients) = self.clients.write() {
            for client in clients.values() {
                if let Ok(client) = client.read() {
                    drop(client.ws.close(CloseCode::Normal));
                    drop(client.ws.shutdown());
                }
            }
            clients.clear();
        }
    }

    fn send(&mut self, id: ClientId, msg_id: MessageId, data: &[u8]) -> Option<Range<usize>> {
        if self.state() != ServerState::Open {
            return None;
        }

        if let Ok(mut clients) = self.clients.write() {
            if let Some(client) = clients.get_mut(&id) {
                let size = data.len();
                let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
                drop(stream.write_u32::<BigEndian>(msg_id.id()));
                drop(stream.write_u32::<BigEndian>(msg_id.version()));
                drop(stream.write(data));
                let data = stream.into_inner();
                if let Ok(client) = client.read() {
                    if client.state == ClientState::Open
                        && client.ws.send(Message::Binary(data)).is_ok()
                    {
                        return Some(0..size);
                    }
                }
            }
        }
        None
    }

    fn send_all(&mut self, id: MessageId, data: &[u8]) {
        if self.state() != ServerState::Open {
            return;
        }
        let size = data.len();
        let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
        drop(stream.write_u32::<BigEndian>(id.id()));
        drop(stream.write_u32::<BigEndian>(id.version()));
        drop(stream.write(data));
        let data = stream.into_inner();
        if let Ok(ws) = self.ws.read() {
            if let Some(ws) = ws.as_ref() {
                ws.send(Message::Binary(data)).unwrap();
            }
        }
    }

    fn read(&mut self) -> Option<MsgData> {
        self.messages.pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        let result = self.messages.iter().cloned().collect();
        self.messages.clear();
        result
    }

    fn process(&mut self) {
        if let Ok(mut clients) = self.clients.write() {
            for client in clients.values() {
                if let Ok(mut client) = client.write() {
                    self.messages.append(&mut client.messages);
                }
            }
            clients.retain(|_, client| {
                if let Ok(client) = client.read() {
                    client.state != ClientState::Closed
                } else {
                    true
                }
            });
            self.clients_ids_cached.clear();
            for id in clients.keys() {
                self.clients_ids_cached.push(*id);
            }
        }
    }
}
