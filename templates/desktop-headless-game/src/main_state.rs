use oxygengine::prelude::*;

const HOST_URL: &str = "127.0.0.1:9009";

// Typical basic game host server that will listen for clients and reply any message it gets.
#[derive(Default)]
pub struct MainState {
    server: Option<ServerId>,
}

impl State for MainState {
    fn on_enter(&mut self, _: &mut Universe) {
        info!("* SERVER START: {}", HOST_URL);
    }

    fn on_exit(&mut self, universe: &mut Universe) {
        info!("* SERVER STOP");
        let mut network = universe.expect_resource_mut::<NetworkHost<DesktopServer>>();
        if let Some(server) = &self.server {
            network.close_server(*server);
        }
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut network = universe.expect_resource_mut::<NetworkHost<DesktopServer>>();
        if self.server.is_none() {
            self.server = network.open_server(HOST_URL);
        }
        if let Some(id) = &self.server {
            if let Some(server) = network.server_mut(*id) {
                for (client_id, message_id, data) in server.read_all() {
                    info!(
                        "* GOT MESSAGE {:?} FROM {:?}:\n{:?}",
                        message_id, client_id, &data
                    );
                    server.send(client_id, message_id, &data);
                }
            }
        }
        StateChange::None
    }
}
