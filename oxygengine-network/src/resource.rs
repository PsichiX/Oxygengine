use crate::{
    client::{Client, ClientID, ClientState},
    server::{Server, ServerID, ServerState},
};
use std::collections::HashMap;

pub struct NetworkHost<S>
where
    S: Server,
{
    servers: HashMap<ServerID, S>,
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
    clients: HashMap<ClientID, C>,
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

    pub fn process(&mut self) {
        for client in self.clients.values_mut() {
            client.process();
        }
        self.clients
            .retain(|_, client| client.state() != ClientState::Closed);
    }
}
