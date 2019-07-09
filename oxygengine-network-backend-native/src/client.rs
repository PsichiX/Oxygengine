use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use network::client::{Client, ClientID, ClientState, MessageID};
use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind, Read, Write},
    mem::replace,
    net::TcpStream,
    ops::Range,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

const STREAM_SLEEP_MS: u64 = 10;

type MsgData = (MessageID, Vec<u8>);

pub struct NativeClient {
    id: ClientID,
    history_size: Arc<Mutex<usize>>,
    state: Arc<Mutex<ClientState>>,
    messages: Arc<Mutex<VecDeque<MsgData>>>,
    thread: Option<JoinHandle<()>>,
    sender: Arc<Mutex<Sender<Vec<u8>>>>,
}

impl Drop for NativeClient {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl NativeClient {
    pub fn history_size(&self) -> usize {
        *self.history_size.lock().unwrap()
    }

    pub fn set_history_size(&mut self, value: usize) {
        *self.history_size.lock().unwrap() = value;
    }

    fn cleanup(&mut self) {
        {
            *self.state.lock().unwrap() = ClientState::Closed;
        }
        let thread = replace(&mut self.thread, None);
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }

    fn read_message(buffer: &[u8], size: usize) -> Option<MsgData> {
        if size >= 8 {
            let mut stream = Cursor::new(buffer);
            let id = stream.read_u32::<BigEndian>().unwrap();
            let version = stream.read_u32::<BigEndian>().unwrap();
            let mut data = Vec::with_capacity(size - 8);
            stream.read_to_end(&mut data).unwrap();
            Some((MessageID::new(id, version), data))
        } else {
            None
        }
    }
}

impl Client for NativeClient {
    fn open(url: &str) -> Option<Self> {
        let id = ClientID::default();
        let url = url.to_owned();
        let state = Arc::new(Mutex::new(ClientState::Connecting));
        let state2 = state.clone();
        let history_size = Arc::new(Mutex::new(0));
        let history_size2 = history_size.clone();
        let messages = Arc::new(Mutex::new(VecDeque::default()));
        let messages2 = messages.clone();
        let (sender, receiver) = channel::<Vec<u8>>();
        let thread = Some(spawn(move || {
            let mut stream = TcpStream::connect(&url).unwrap();
            stream.set_nonblocking(true).expect(&format!(
                "Client {:?} cannot set non-blocking streaming on: {}",
                id, &url
            ));
            stream.set_nodelay(true).expect(&format!(
                "Client {:?} cannot set no-delay streaming on: {}",
                id, &url,
            ));
            {
                *state2.lock().unwrap() = ClientState::Open;
            }
            let mut receive_buffer = vec![0; 1024];
            loop {
                if *state2.lock().unwrap() == ClientState::Closed {
                    break;
                }
                loop {
                    match stream.read_to_end(&mut receive_buffer) {
                        Ok(size) => {
                            if let Some(msg) = Self::read_message(&receive_buffer, size) {
                                let mut messages = messages2.lock().unwrap();
                                messages.push_back(msg);
                                let history_size = *history_size2.lock().unwrap();
                                if history_size > 0 {
                                    while messages.len() > history_size {
                                        messages.pop_front();
                                    }
                                }
                            }
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => panic!("Client {:?} reading {} got IO error: {}", id, &url, e),
                    };
                }
                while let Ok(data) = receiver.try_recv() {
                    stream.write_all(&data).unwrap();
                }
                sleep(Duration::from_millis(STREAM_SLEEP_MS));
            }
            {
                *state2.lock().unwrap() = ClientState::Closed;
            }
        }));
        Some(Self {
            id,
            history_size,
            state,
            messages,
            thread,
            sender: Arc::new(Mutex::new(sender)),
        })
    }

    fn close(mut self) -> Self {
        self.cleanup();
        self
    }

    fn id(&self) -> ClientID {
        self.id
    }

    fn state(&self) -> ClientState {
        *self.state.lock().unwrap()
    }

    fn send(&mut self, id: MessageID, data: &[u8]) -> Option<Range<usize>> {
        if self.state() == ClientState::Open {
            let size = data.len();
            let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
            drop(stream.write_u32::<BigEndian>(id.id()));
            drop(stream.write_u32::<BigEndian>(id.version()));
            drop(stream.write(data));
            let data = stream.into_inner();
            if self.sender.lock().unwrap().send(data).is_ok() {
                return Some(0..size);
            }
        }
        None
    }

    fn read(&mut self) -> Option<MsgData> {
        self.messages.lock().unwrap().pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        let mut messages = self.messages.lock().unwrap();
        let result = messages.iter().cloned().collect();
        messages.clear();
        result
    }
}
