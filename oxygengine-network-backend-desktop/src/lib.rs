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
    client::{ClientID, ClientState, MessageID},
    server::{Server, ServerID, ServerState},
};
use std::{
    collections::{HashMap, VecDeque},
    io::{Cursor, Read, Write},
    mem::replace,
    ops::{DerefMut, Range},
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
};
use ws::{CloseCode, Handler, Handshake, Message, Result, Sender as WsSender, WebSocket};

type MsgData = (ClientID, MessageID, Vec<u8>);

#[derive(Clone)]
struct Client {
    id: ClientID,
    ws: WsSender,
    state: ClientState,
    messages: VecDeque<MsgData>,
}

struct ClientHandler {
    client: Arc<Mutex<Client>>,
}

impl Handler for ClientHandler {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.client.lock().unwrap().state = ClientState::Open;
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
                let mut client = self.client.lock().unwrap();
                let client_id = client.id;
                client
                    .messages
                    .push_back((client_id, MessageID::new(id, version), data));
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut client = self.client.lock().unwrap();
        client.state = ClientState::Closed;
    }
}

pub struct DesktopServer {
    id: ServerID,
    state: Arc<Mutex<ServerState>>,
    clients: Arc<Mutex<HashMap<ClientID, Arc<Mutex<Client>>>>>,
    clients_ids_cached: Vec<ClientID>,
    messages: VecDeque<MsgData>,
    ws: Arc<Mutex<Option<WsSender>>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for DesktopServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl DesktopServer {
    fn cleanup(&mut self) {
        *self.state.lock().unwrap() = ServerState::Closed;
        if let Some(ws) = self.ws.lock().unwrap().deref_mut() {
            ws.shutdown().unwrap();
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
        let state = Arc::new(Mutex::new(ServerState::Starting));
        let state2 = state.clone();
        let clients = Arc::new(Mutex::new(HashMap::default()));
        let clients2 = clients.clone();
        let sender = Arc::new(Mutex::new(None));
        let sender2 = sender.clone();
        let thread = Some(spawn(move || {
            let ws = WebSocket::new(|ws| {
                let id = ClientID::default();
                let client = Arc::new(Mutex::new(Client {
                    id,
                    ws,
                    state: ClientState::Connecting,
                    messages: VecDeque::new(),
                }));
                clients2.lock().unwrap().insert(id, client.clone());
                ClientHandler { client }
            })
            .unwrap();
            {
                *sender2.lock().unwrap() = Some(ws.broadcaster());
                *state2.lock().unwrap() = ServerState::Open;
            }
            ws.listen(&url).unwrap();
            {
                *sender2.lock().unwrap() = None;
                *state2.lock().unwrap() = ServerState::Closed;
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
            let client = client.lock().unwrap();
            client.ws.close(CloseCode::Normal).unwrap();
            client.ws.shutdown().unwrap();
        }
        clients.remove(&id);
    }

    fn disconnect_all(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        for client in clients.values() {
            let client = client.lock().unwrap();
            client.ws.close(CloseCode::Normal).unwrap();
            client.ws.shutdown().unwrap();
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
            let client = client.lock().unwrap();
            if client.state == ClientState::Open && client.ws.send(Message::Binary(data)).is_ok() {
                return Some(0..size);
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
        if let Some(ws) = self.ws.lock().unwrap().deref_mut() {
            ws.send(Message::Binary(data)).unwrap();
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
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|_, client| client.lock().unwrap().state != ClientState::Closed);
        self.clients_ids_cached.clear();
        for id in clients.keys() {
            self.clients_ids_cached.push(*id);
        }
        for client in clients.values() {
            self.messages.append(&mut client.lock().unwrap().messages);
        }
    }
}
