use crate::client::{ClientId, MessageId};
use core::id::ID;
use std::ops::Range;

pub type ServerId = ID<()>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerState {
    Starting,
    Open,
    Closed,
}

pub trait Server: Send + Sync + Sized {
    fn open(url: &str) -> Option<Self>;

    fn close(self) -> Self;

    fn id(&self) -> ServerId;

    fn state(&self) -> ServerState;

    fn clients(&self) -> &[ClientId];

    fn disconnect(&mut self, id: ClientId);

    fn disconnect_all(&mut self);

    fn send(&mut self, id: ClientId, msg_id: MessageId, data: &[u8]) -> Option<Range<usize>>;

    fn send_all(&mut self, id: MessageId, data: &[u8]);

    fn read(&mut self) -> Option<(ClientId, MessageId, Vec<u8>)>;

    fn read_all(&mut self) -> Vec<(ClientId, MessageId, Vec<u8>)> {
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

    fn id(&self) -> ServerId {
        Default::default()
    }

    fn state(&self) -> ServerState {
        ServerState::Closed
    }

    fn clients(&self) -> &[ClientId] {
        &[]
    }

    fn disconnect(&mut self, _: ClientId) {}

    fn disconnect_all(&mut self) {}

    fn send(&mut self, _: ClientId, _: MessageId, _: &[u8]) -> Option<Range<usize>> {
        None
    }

    fn send_all(&mut self, _: MessageId, _: &[u8]) {}

    fn read(&mut self) -> Option<(ClientId, MessageId, Vec<u8>)> {
        None
    }

    fn read_all(&mut self) -> Vec<(ClientId, MessageId, Vec<u8>)> {
        vec![]
    }
}
