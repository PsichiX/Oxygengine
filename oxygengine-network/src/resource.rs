use crate::{
    client::{Client, ClientID, ClientState, MessageID},
    server::{Server, ServerID, ServerState},
};
use std::{collections::HashMap, ops::Range};

pub struct NetworkHost<S>
where
    S: Server,
{
    servers: HashMap<ServerID, S>,
    messages: HashMap<ServerID, Vec<(ClientID, MessageID, Vec<u8>)>>,
}

impl<S> Default for NetworkHost<S>
where
    S: Server,
{
    fn default() -> Self {
        Self {
            servers: Default::default(),
            messages: Default::default(),
        }
    }
}

impl<S> NetworkHost<S>
where
    S: Server,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_server(&mut self, url: &str) -> Option<ServerID> {
        if let Some(server) = S::open(url) {
            let id = server.id();
            self.servers.insert(id, server);
            Some(id)
        } else {
            None
        }
    }

    pub fn close_server(&mut self, id: ServerID) -> bool {
        if let Some(server) = self.servers.remove(&id) {
            server.close();
            true
        } else {
            false
        }
    }

    pub fn server(&self, id: ServerID) -> Option<&S> {
        self.servers.get(&id)
    }

    pub fn server_mut(&mut self, id: ServerID) -> Option<&mut S> {
        self.servers.get_mut(&id)
    }

    pub fn has_server(&self, id: ServerID) -> bool {
        self.servers.contains_key(&id)
    }

    pub fn send_all(&mut self, id: ServerID, msg_id: MessageID, data: &[u8]) {
        if let Some(server) = self.servers.get_mut(&id) {
            server.send_all(msg_id, data);
        }
    }

    pub fn process(&mut self) {
        self.messages.clear();
        for (id, server) in &mut self.servers {
            server.listen();
            if server.state() == ServerState::Open {
                let mut messages = vec![];
                while let Some((client, mid, data)) = server.receive_all() {
                    messages.push((client, mid, data));
                }
                self.messages.insert(*id, messages);
            }
        }
        self.servers
            .retain(|_, server| server.state() != ServerState::Closed);
    }
}

pub struct Network<C>
where
    C: Client,
{
    version: u32,
    clients: HashMap<ClientID, C>,
    messages: HashMap<ClientID, Vec<(MessageID, Vec<u8>)>>,
}

impl<C> Network<C>
where
    C: Client,
{
    pub fn new(version: u32) -> Self {
        Self {
            version,
            clients: Default::default(),
            messages: Default::default(),
        }
    }

    pub fn open_client(&mut self, url: &str) -> Option<ClientID> {
        if let Some(client) = C::open(url) {
            let id = client.id();
            self.clients.insert(id, client);
            Some(id)
        } else {
            None
        }
    }

    pub fn close_client(&mut self, id: ClientID) -> bool {
        if let Some(client) = self.clients.remove(&id) {
            client.close();
            true
        } else {
            false
        }
    }

    pub fn client(&self, id: ClientID) -> Option<&C> {
        self.clients.get(&id)
    }

    pub fn client_mut(&mut self, id: ClientID) -> Option<&mut C> {
        self.clients.get_mut(&id)
    }

    pub fn has_client(&self, id: ClientID) -> bool {
        self.clients.contains_key(&id)
    }

    pub fn send(&mut self, id: ClientID, msg_id: u32, data: &[u8]) -> Option<Range<usize>> {
        if let Some(client) = self.clients.get_mut(&id) {
            client.send((msg_id, self.version).into(), data)
        } else {
            None
        }
    }

    pub fn read(&self, id: ClientID) -> Option<impl Iterator<Item = (MessageID, &[u8])>> {
        if let Some(client) = self.messages.get(&id) {
            if !client.is_empty() {
                return Some(client.iter().map(|(mid, data)| (*mid, data.as_slice())));
            }
        }
        None
    }

    pub fn process(&mut self) {
        self.messages.clear();
        let version = self.version;
        for (id, client) in &mut self.clients {
            if client.state() == ClientState::Open {
                let mut messages = vec![];
                while let Some((mid, data)) = client.receive() {
                    if mid.version() == version {
                        messages.push((mid, data));
                    }
                }
                self.messages.insert(*id, messages);
            }
        }
        self.clients
            .retain(|_, client| client.state() != ClientState::Closed);
    }
}
