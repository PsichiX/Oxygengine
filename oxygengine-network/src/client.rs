use core::id::ID;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageID(u32, u32);

impl MessageID {
    pub fn new(id: u32, version: u32) -> Self {
        Self(id, version)
    }

    pub fn id(self) -> u32 {
        self.0
    }

    pub fn version(self) -> u32 {
        self.1
    }
}

impl From<(u32, u32)> for MessageID {
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[u32; 2]> for MessageID {
    fn from(value: [u32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

pub type ClientID = ID<()>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClientState {
    Connecting,
    Open,
    Closed,
}

pub trait Client: Send + Sync + Sized {
    fn open(url: &str) -> Option<Self>;

    fn close(self) -> Self;

    fn id(&self) -> ClientID;

    fn state(&self) -> ClientState;

    fn send(&mut self, id: MessageID, data: &[u8]) -> Option<Range<usize>>;

    fn read(&mut self) -> Option<(MessageID, Vec<u8>)>;

    fn read_all(&mut self) -> Vec<(MessageID, Vec<u8>)> {
        let mut result = vec![];
        while let Some(msg) = self.read() {
            result.push(msg);
        }
        result
    }

    fn process(&mut self) {}
}

impl Client for () {
    fn open(_: &str) -> Option<Self> {
        Some(())
    }

    fn close(self) -> Self {
        self
    }

    fn id(&self) -> ClientID {
        Default::default()
    }

    fn state(&self) -> ClientState {
        ClientState::Closed
    }

    fn send(&mut self, _: MessageID, _: &[u8]) -> Option<Range<usize>> {
        None
    }

    fn read(&mut self) -> Option<(MessageID, Vec<u8>)> {
        None
    }

    fn read_all(&mut self) -> Vec<(MessageID, Vec<u8>)> {
        vec![]
    }
}
