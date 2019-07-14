use crate::utils::DoOnDrop;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use network::client::{Client, ClientID, ClientState, MessageID};
use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind, Read, Write},
    mem::replace,
    net::{Shutdown, TcpStream},
    ops::Range,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{sleep, Builder as ThreadBuilder, JoinHandle},
    time::Duration,
};

const STREAM_SLEEP_MS: u64 = 10;

type MsgData = (MessageID, Vec<u8>);

pub struct NativeClient {
    id: ClientID,
    history_size: Arc<AtomicUsize>,
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
        self.history_size.load(Ordering::Relaxed)
    }

    pub fn set_history_size(&mut self, value: usize) {
        self.history_size.store(value, Ordering::Relaxed);
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

    fn read_message(buffer: &[u8]) -> (MessageID, usize) {
        let mut stream = Cursor::new(buffer);
        let id = stream.read_u32::<BigEndian>().unwrap();
        let version = stream.read_u32::<BigEndian>().unwrap();
        let size = stream.read_u32::<BigEndian>().unwrap();
        (MessageID::new(id, version), size as usize)
    }
}

impl From<TcpStream> for NativeClient {
    fn from(mut stream: TcpStream) -> Self {
        let id = ClientID::default();
        let url = stream.peer_addr().unwrap().to_string();
        let state = Arc::new(Mutex::new(ClientState::Connecting));
        let state2 = state.clone();
        let history_size = Arc::new(AtomicUsize::new(0));
        let history_size2 = history_size.clone();
        let messages = Arc::new(Mutex::new(VecDeque::<MsgData>::default()));
        let messages2 = messages.clone();
        let (sender, receiver) = channel::<Vec<u8>>();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Client: {:?}", id))
                .spawn(move || {
                    let state3 = state2.clone();
                    let _ = DoOnDrop::new(move || *state3.lock().unwrap() = ClientState::Closed);
                    stream.set_nonblocking(true).unwrap_or_else(|_| {
                        panic!(
                            "Client {:?} cannot set non-blocking streaming on: {}",
                            id, &url
                        )
                    });
                    stream.set_nodelay(true).unwrap_or_else(|_| {
                        panic!("Client {:?} cannot set no-delay streaming on: {}", id, &url,)
                    });
                    {
                        *state2.lock().unwrap() = ClientState::Open;
                    }
                    let mut header = vec![0; 12];
                    let mut left_to_read: Option<(MessageID, usize, Vec<u8>)> = None;
                    'main: loop {
                        if *state2.lock().unwrap() == ClientState::Closed {
                            break;
                        }
                        loop {
                            let reset =
                                if let Some((lfr_msg, lfr_size, lfr_buff)) = &mut left_to_read {
                                    let mut buffer = vec![0; *lfr_size];
                                    match stream.read(&mut buffer) {
                                        Ok(size) => {
                                            lfr_buff.extend_from_slice(&buffer[0..size]);
                                            if size >= *lfr_size {
                                                let mut messages = messages2.lock().unwrap();
                                                messages.push_back((*lfr_msg, lfr_buff.clone()));
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            break;
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                            break 'main;
                                        }
                                        Err(e) => panic!(
                                            "Client {:?} reading body {} got IO error: {}",
                                            id, &url, e
                                        ),
                                    }
                                } else {
                                    match stream.read_exact(&mut header) {
                                        Ok(()) => {
                                            let (msg, size) = Self::read_message(&header);
                                            if size > 0 {
                                                left_to_read = Some((msg, size, vec![]));
                                            } else {
                                                let mut messages = messages2.lock().unwrap();
                                                messages.push_back((msg, vec![]));
                                            }
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            break;
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                            break 'main;
                                        }
                                        Err(e) => panic!(
                                            "Client {:?} reading header {} got IO error: {}",
                                            id, &url, e
                                        ),
                                    }
                                    false
                                };
                            if reset {
                                left_to_read = None;
                            }
                        }
                        {
                            let history_size = history_size2.load(Ordering::Relaxed);
                            if history_size > 0 {
                                let mut messages = messages2.lock().unwrap();
                                while messages.len() > history_size {
                                    messages.pop_front();
                                }
                            }
                        }
                        while let Ok(data) = receiver.try_recv() {
                            stream.write_all(&data).unwrap();
                        }
                        sleep(Duration::from_millis(STREAM_SLEEP_MS));
                    }
                    {
                        *state2.lock().unwrap() = ClientState::Closed;
                    }
                })
                .unwrap(),
        );
        Self {
            id,
            history_size,
            state,
            messages,
            thread,
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

impl Client for NativeClient {
    fn open(url: &str) -> Option<Self> {
        let id = ClientID::default();
        let url = url.to_owned();
        let state = Arc::new(Mutex::new(ClientState::Connecting));
        let state2 = state.clone();
        let history_size = Arc::new(AtomicUsize::new(0));
        let history_size2 = history_size.clone();
        let messages = Arc::new(Mutex::new(VecDeque::<MsgData>::default()));
        let messages2 = messages.clone();
        let (sender, receiver) = channel::<Vec<u8>>();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Client: {:?}", id))
                .spawn(move || {
                    let state3 = state2.clone();
                    let _ = DoOnDrop::new(move || *state3.lock().unwrap() = ClientState::Closed);
                    let mut stream = TcpStream::connect(&url)
                        .unwrap_or_else(|_| panic!("Client {:?} cannot connect to: {}", id, &url));
                    stream.set_nonblocking(true).unwrap_or_else(|_| {
                        panic!(
                            "Client {:?} cannot set non-blocking streaming on: {}",
                            id, &url
                        )
                    });
                    stream.set_nodelay(true).unwrap_or_else(|_| {
                        panic!("Client {:?} cannot set no-delay streaming on: {}", id, &url,)
                    });
                    {
                        *state2.lock().unwrap() = ClientState::Open;
                    }
                    let mut header = vec![0; 12];
                    let mut left_to_read: Option<(MessageID, usize, Vec<u8>)> = None;
                    'main: loop {
                        if *state2.lock().unwrap() == ClientState::Closed {
                            break;
                        }
                        loop {
                            let reset =
                                if let Some((lfr_msg, lfr_size, lfr_buff)) = &mut left_to_read {
                                    let mut buffer = vec![0; *lfr_size];
                                    match stream.read(&mut buffer) {
                                        Ok(size) => {
                                            lfr_buff.extend_from_slice(&buffer[0..size]);
                                            if size >= *lfr_size {
                                                let mut messages = messages2.lock().unwrap();
                                                messages.push_back((*lfr_msg, lfr_buff.clone()));
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            break;
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                            break 'main;
                                        }
                                        Err(e) => panic!(
                                            "Client {:?} reading body {} got IO error: {}",
                                            id, &url, e
                                        ),
                                    }
                                } else {
                                    match stream.read_exact(&mut header) {
                                        Ok(()) => {
                                            let (msg, size) = Self::read_message(&header);
                                            if size > 0 {
                                                left_to_read = Some((msg, size, vec![]));
                                            } else {
                                                let mut messages = messages2.lock().unwrap();
                                                messages.push_back((msg, vec![]));
                                            }
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            break;
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                            break 'main;
                                        }
                                        Err(e) => panic!(
                                            "Client {:?} reading header {} got IO error: {}",
                                            id, &url, e
                                        ),
                                    }
                                    false
                                };
                            if reset {
                                left_to_read = None;
                            }
                        }
                        {
                            let history_size = history_size2.load(Ordering::Relaxed);
                            if history_size > 0 {
                                let mut messages = messages2.lock().unwrap();
                                while messages.len() > history_size {
                                    messages.pop_front();
                                }
                            }
                        }
                        while let Ok(data) = receiver.try_recv() {
                            stream.write_all(&data).unwrap();
                        }
                        sleep(Duration::from_millis(STREAM_SLEEP_MS));
                    }
                    stream.shutdown(Shutdown::Both).unwrap();
                    {
                        *state2.lock().unwrap() = ClientState::Closed;
                    }
                })
                .unwrap(),
        );
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
            let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 12));
            drop(stream.write_u32::<BigEndian>(id.id()));
            drop(stream.write_u32::<BigEndian>(id.version()));
            drop(stream.write_u32::<BigEndian>(size as u32));
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
        self.messages.lock().unwrap().drain(..).collect()
    }
}
