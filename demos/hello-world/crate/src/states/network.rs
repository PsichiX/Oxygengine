use crate::macros::*;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct NetworkState {
    client: Option<ClientID>,
}

impl State for NetworkState {
    fn on_enter(&mut self, world: &mut World) {
        let url = "ws://localhost:8090";
        self.client = world
            .write_resource::<Network<WebClient>>()
            .open_client(url);
        console_log!("OPEN CLIENT: {} => {:?}", url, self.client);
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let Some(client) = self.client {
            let network = &world.read_resource::<Network<WebClient>>();
            if !network.has_client(client) {
                console_log!("CLIENT DISCONNECTED: {:?}", client);
                return StateChange::Pop;
            }
            drop(if let Some(messages) = network.read(client) {
                let messages = messages.collect::<Vec<_>>();
                console_log!("READ MESSAGES: {:#?}", messages);
            });
        }
        StateChange::None
    }
}
