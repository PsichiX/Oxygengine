use crate::{
    client::{Client, ClientId, ClientState},
    server::{Server, ServerId, ServerState},
};
use std::collections::HashMap;

pub struct NetworkHost<S>
where
    S: Server,
{
    servers: HashMap<ServerId, S>,
}

impl<S> Default for NetworkHost<S>
where
    S: Server,
{
    fn default() -> Self {
        Self {
            servers: Default::default(),
        }
    }
}

impl<S> NetworkHost<S>
where
    S: Server,
{
    pub fn open_server(&mut self, url: &str) -> Option<ServerId> {
        if let Some(server) = S::open(url) {
            let id = server.id();
            self.servers.insert(id, server);
            Some(id)
        } else {
            None
        }
    }

    pub fn close_server(&mut self, id: ServerId) -> bool {
        if let Some(server) = self.servers.remove(&id) {
            server.close();
            true
        } else {
            false
        }
    }

    pub fn server(&self, id: ServerId) -> Option<&S> {
        self.servers.get(&id)
    }

    pub fn server_mut(&mut self, id: ServerId) -> Option<&mut S> {
        self.servers.get_mut(&id)
    }

    pub fn has_server(&self, id: ServerId) -> bool {
        self.servers.contains_key(&id)
    }

    pub fn process(&mut self) {
        for server in self.servers.values_mut() {
            server.process();
        }
        self.servers
            .retain(|_, server| server.state() != ServerState::Closed);
    }
}

pub struct Network<C>
where
    C: Client,
{
    clients: HashMap<ClientId, C>,
}

impl<C> Default for Network<C>
where
    C: Client,
{
    fn default() -> Self {
        Self {
            clients: Default::default(),
        }
    }
}

impl<C> Network<C>
where
    C: Client,
{
    pub fn open_client(&mut self, url: &str) -> Option<ClientId> {
        if let Some(client) = C::open(url) {
            let id = client.id();
            self.clients.insert(id, client);
            Some(id)
        } else {
            None
        }
    }

    pub fn close_client(&mut self, id: ClientId) -> bool {
        if let Some(client) = self.clients.remove(&id) {
            client.close();
            true
        } else {
            false
        }
    }

    pub fn client(&self, id: ClientId) -> Option<&C> {
        self.clients.get(&id)
    }

    pub fn client_mut(&mut self, id: ClientId) -> Option<&mut C> {
        self.clients.get_mut(&id)
    }

    pub fn has_client(&self, id: ClientId) -> bool {
        self.clients.contains_key(&id)
    }

    pub fn process(&mut self) {
        for client in self.clients.values_mut() {
            client.process();
        }
        self.clients
            .retain(|_, client| client.state() != ClientState::Closed);
    }
}
