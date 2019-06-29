use crate::client::{ClientID, MessageID};
use core::id::ID;
use std::ops::Range;

pub type ServerID = ID<()>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerState {
    Starting,
    Open,
    Closed,
}

pub trait Server: Send + Sync + Sized {
    fn open(url: &str) -> Option<Self>;

    fn close(self) -> Self;

    fn id(&self) -> ServerID;

    fn state(&self) -> ServerState;

    fn clients(&self) -> &[ClientID];

    fn disconnect(&mut self, id: ClientID);

    fn disconnect_all(&mut self);

    fn send(&mut self, id: ClientID, msg_id: MessageID, data: &[u8]) -> Option<Range<usize>>;

    fn send_all(&mut self, id: MessageID, data: &[u8]);

    fn read(&mut self) -> Option<(ClientID, MessageID, Vec<u8>)>;

    fn read_all(&mut self) -> Vec<(ClientID, MessageID, Vec<u8>)> {
        let mut result = vec![];
        while let Some(msg) = self.read() {
            result.push(msg);
        }
        result
    }

    fn process(&mut self) {}
}

impl Server for () {
    fn open(_: &str) -> Option<Self> {
        Some(())
    }

    fn close(self) -> Self {
        self
    }

    fn id(&self) -> ServerID {
        Default::default()
    }

    fn state(&self) -> ServerState {
        ServerState::Closed
    }

    fn clients(&self) -> &[ClientID] {
        &[]
    }

    fn disconnect(&mut self, _: ClientID) {}

    fn disconnect_all(&mut self) {}

    fn send(&mut self, _: ClientID, _: MessageID, _: &[u8]) -> Option<Range<usize>> {
        None
    }

    fn send_all(&mut self, _: MessageID, _: &[u8]) {}

    fn read(&mut self) -> Option<(ClientID, MessageID, Vec<u8>)> {
        None
    }

    fn read_all(&mut self) -> Vec<(ClientID, MessageID, Vec<u8>)> {
        vec![]
    }
}
