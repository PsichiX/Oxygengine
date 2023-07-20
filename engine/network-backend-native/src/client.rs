use crate::utils::DoOnDrop;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use network::client::{Client, ClientId, ClientState, MessageId};
use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind, Read, Write},
    net::{Shutdown, TcpStream},
    ops::Range,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex, RwLock,
    },
    thread::{sleep, Builder as ThreadBuilder, JoinHandle},
    time::Duration,
};

const STREAM_SLEEP_MS: u64 = 10;

type MsgData = (MessageId, Vec<u8>);

pub struct NativeClient {
    id: ClientId,
    history_size: Arc<AtomicUsize>,
    state: Arc<RwLock<ClientState>>,
    messages: Arc<RwLock<VecDeque<MsgData>>>,
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
        if let Ok(mut state) = self.state.write() {
            *state = ClientState::Closed;
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }

    fn read_message(buffer: &[u8]) -> (MessageId, usize) {
        let mut stream = Cursor::new(buffer);
        let id = stream.read_u32::<BigEndian>().unwrap();
        let version = stream.read_u32::<BigEndian>().unwrap();
        let size = stream.read_u32::<BigEndian>().unwrap();
        (MessageId::new(id, version), size as usize)
    }
}

impl From<TcpStream> for NativeClient {
    fn from(mut stream: TcpStream) -> Self {
        let id = ClientId::default();
        let url = stream.peer_addr().unwrap().to_string();
        let state = Arc::new(RwLock::new(ClientState::Connecting));
        let state2 = state.clone();
        let history_size = Arc::new(AtomicUsize::new(0));
        let history_size2 = history_size.clone();
        let messages = Arc::new(RwLock::new(VecDeque::<MsgData>::default()));
        let messages2 = messages.clone();
        let (sender, receiver) = channel::<Vec<u8>>();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Client: {:?}", id))
                .spawn(move || {
                    let state3 = state2.clone();
                    let _ = DoOnDrop::new(move || {
                        if let Ok(mut state) = state3.write() {
                            *state = ClientState::Closed;
                        }
                    });
                    stream.set_nonblocking(true).unwrap_or_else(|_| {
                        panic!(
                            "Client {:?} cannot set non-blocking streaming on: {}",
                            id, &url
                        )
                    });
                    stream.set_nodelay(true).unwrap_or_else(|_| {
                        panic!("Client {:?} cannot set no-delay streaming on: {}", id, &url,)
                    });
                    if let Ok(mut state) = state2.write() {
                        *state = ClientState::Open;
                    }
                    let mut header = vec![0; 12];
                    let mut left_to_read: Option<(MessageId, usize, Vec<u8>)> = None;
                    'main: loop {
                        if let Ok(state) = state2.read() {
                            if *state == ClientState::Closed {
                                break;
                            }
                        }
                        loop {
                            let reset = if let Some((lfr_msg, lfr_size, lfr_buff)) =
                                &mut left_to_read
                            {
                                let mut buffer = vec![0; *lfr_size];
                                match stream.read(&mut buffer) {
                                    Ok(size) => {
                                        lfr_buff.extend_from_slice(&buffer[0..size]);
                                        if size >= *lfr_size {
                                            if let Ok(mut messages) = messages2.write() {
                                                messages.push_back((*lfr_msg, lfr_buff.clone()));
                                            }
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
                                        } else if let Ok(mut messages) = messages2.write() {
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
                                if let Ok(mut messages) = messages2.write() {
                                    while messages.len() > history_size {
                                        messages.pop_front();
                                    }
                                }
                            }
                        }
                        while let Ok(data) = receiver.try_recv() {
                            stream.write_all(&data).unwrap();
                        }
                        sleep(Duration::from_millis(STREAM_SLEEP_MS));
                    }
                    if let Ok(mut state) = state2.write() {
                        *state = ClientState::Closed;
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
        let id = ClientId::default();
        let url = url.to_owned();
        let state = Arc::new(RwLock::new(ClientState::Connecting));
        let state2 = state.clone();
        let history_size = Arc::new(AtomicUsize::new(0));
        let history_size2 = history_size.clone();
        let messages = Arc::new(RwLock::new(VecDeque::<MsgData>::default()));
        let messages2 = messages.clone();
        let (sender, receiver) = channel::<Vec<u8>>();
        let thread = Some(
            ThreadBuilder::new()
                .name(format!("Client: {:?}", id))
                .spawn(move || {
                    let state3 = state2.clone();
                    let _ = DoOnDrop::new(move || {
                        if let Ok(mut state) = state3.write() {
                            *state = ClientState::Closed;
                        }
                    });
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
                    if let Ok(mut state) = state2.write() {
                        *state = ClientState::Open;
                    }
                    let mut header = vec![0; 12];
                    let mut left_to_read: Option<(MessageId, usize, Vec<u8>)> = None;
                    'main: loop {
                        if let Ok(state) = state2.read() {
                            if *state == ClientState::Closed {
                                break;
                            }
                        }
                        loop {
                            let reset = if let Some((lfr_msg, lfr_size, lfr_buff)) =
                                &mut left_to_read
                            {
                                let mut buffer = vec![0; *lfr_size];
                                match stream.read(&mut buffer) {
                                    Ok(size) => {
                                        lfr_buff.extend_from_slice(&buffer[0..size]);
                                        if size >= *lfr_size {
                                            if let Ok(mut messages) = messages2.write() {
                                                messages.push_back((*lfr_msg, lfr_buff.clone()));
                                            }
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
                                        } else if let Ok(mut messages) = messages2.write() {
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
                                if let Ok(mut messages) = messages2.write() {
                                    while messages.len() > history_size {
                                        messages.pop_front();
                                    }
                                }
                            }
                        }
                        while let Ok(data) = receiver.try_recv() {
                            stream.write_all(&data).unwrap();
                        }
                        sleep(Duration::from_millis(STREAM_SLEEP_MS));
                    }
                    stream.shutdown(Shutdown::Both).unwrap();
                    if let Ok(mut state) = state2.write() {
                        *state = ClientState::Closed;
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

    fn id(&self) -> ClientId {
        self.id
    }

    fn state(&self) -> ClientState {
        if let Ok(state) = self.state.read() {
            *state
        } else {
            ClientState::default()
        }
    }

    fn send(&mut self, id: MessageId, data: &[u8]) -> Option<Range<usize>> {
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
        if let Ok(mut messages) = self.messages.write() {
            messages.pop_front()
        } else {
            None
        }
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        if let Ok(mut messages) = self.messages.write() {
            messages.drain(..).collect()
        } else {
            vec![]
        }
    }
}
