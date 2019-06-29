use crate::consts::{HOST_URL, MSG_TEXT, VERSION};
use bincode::deserialize;
use oxygengine::prelude::*;
use std::collections::VecDeque;

pub struct MainState {
    server: Option<ServerID>,
    history: VecDeque<Vec<u8>>,
    history_capacity: usize,
    users: Vec<ClientID>,
}

impl MainState {
    pub fn new(history_capacity: usize) -> Self {
        Self {
            server: None,
            history: Default::default(),
            history_capacity,
            users: vec![],
        }
    }

    fn store_message(&mut self, content: Vec<u8>) {
        self.history.push_back(content);
        while self.history.len() > self.history_capacity && self.history.pop_front().is_some() {}
    }
}

impl State for MainState {
    fn on_enter(&mut self, _: &mut World) {
        info!("* SERVER START");
    }

    fn on_exit(&mut self, world: &mut World) {
        info!("* SERVER STOP");
        let network = &mut world.write_resource::<NetworkHost<DesktopServer>>();
        if let Some(server) = &self.server {
            network.close_server(*server);
        }
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let network = &mut world.write_resource::<NetworkHost<DesktopServer>>();
        if self.server.is_none() {
            self.server = network.open_server(HOST_URL);
        }
        if let Some(id) = &self.server {
            if let Some(server) = network.server_mut(*id) {
                let text_message = MessageID::new(MSG_TEXT, VERSION);
                let clients = server.clients().to_vec();
                for client in &clients {
                    if !self.users.contains(client) {
                        info!("* CLIENT CONNECTED: {:?}", client);
                        self.users.push(*client);
                        for msg in &self.history {
                            server.send(*client, text_message, msg);
                        }
                    }
                }
                self.users.retain(|id| {
                    let status = clients.contains(id);
                    if !status {
                        info!("* CLIENT DISCONNECTED: {:?}", id);
                    }
                    status
                });

                let messages = server.read_all();
                for (cid, mid, data) in messages {
                    if mid.id() == MSG_TEXT {
                        server.send_all(mid, &data);
                        if let Ok(msg) = deserialize::<String>(&data) {
                            info!("* MESSAGE FROM: {:?} | {:?}", cid, msg);
                        }
                        self.store_message(data.to_vec());
                    }
                }
            }
        }
        StateChange::None
    }
}
