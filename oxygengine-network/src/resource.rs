use crate::client::{Client, ClientID, ClientState, MessageID};
use std::{collections::HashMap, ops::Range};

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
