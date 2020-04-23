extern crate byteorder;
extern crate oxygengine_core as core;
extern crate oxygengine_network as network;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use js_sys::*;
use network::client::{Client, ClientID, ClientState, MessageID};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    io::{Cursor, Write},
    ops::Range,
    rc::Rc,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub mod prelude {
    pub use crate::*;
}

type MsgData = (MessageID, Vec<u8>);

#[derive(Clone)]
pub struct WebClient {
    socket: WebSocket,
    id: ClientID,
    history_size: Rc<Cell<usize>>,
    state: Rc<Cell<ClientState>>,
    #[allow(clippy::type_complexity)]
    messages: Rc<RefCell<VecDeque<MsgData>>>,
}

unsafe impl Send for WebClient {}
unsafe impl Sync for WebClient {}

impl WebClient {
    pub fn history_size(&self) -> usize {
        self.history_size.get()
    }

    pub fn set_history_size(&mut self, value: usize) {
        self.history_size.set(value);
    }
}

impl Client for WebClient {
    fn open(url: &str) -> Option<Self> {
        if let Ok(socket) = WebSocket::new(url) {
            socket.set_binary_type(BinaryType::Arraybuffer);
            let history_size = Rc::new(Cell::new(0));
            let state = Rc::new(Cell::new(ClientState::Connecting));
            let messages = Rc::new(RefCell::new(Default::default()));
            {
                let state2 = state.clone();
                let closure = Closure::wrap(Box::new(move |_: Event| {
                    state2.set(ClientState::Open);
                }) as Box<dyn FnMut(_)>);
                socket.set_onopen(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            {
                let state2 = state.clone();
                let closure = Closure::wrap(Box::new(move |_: Event| {
                    state2.set(ClientState::Closed);
                }) as Box<dyn FnMut(_)>);
                socket.set_onclose(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            {
                let history_size2 = history_size.clone();
                let messages2 = messages.clone();
                let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
                    let buff = event.data();
                    if buff.is_instance_of::<ArrayBuffer>() {
                        let typebuf: Uint8Array = Uint8Array::new(&buff);
                        let mut body = vec![0; typebuf.length() as usize];
                        typebuf.copy_to(&mut body[..]);
                        let mut stream = Cursor::new(body);
                        if let Ok(id) = stream.read_u32::<BigEndian>() {
                            if let Ok(version) = stream.read_u32::<BigEndian>() {
                                let id = MessageID::new(id, version);
                                let data = stream.into_inner()[8..].to_vec();
                                let messages: &mut VecDeque<_> = &mut messages2.borrow_mut();
                                messages.push_back((id, data));
                                let history_size = history_size2.get();
                                if history_size > 0 {
                                    while messages.len() > history_size {
                                        messages.pop_front();
                                    }
                                }
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                socket.set_onmessage(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            Some(Self {
                socket,
                id: Default::default(),
                history_size,
                state,
                messages,
            })
        } else {
            None
        }
    }

    fn close(self) -> Self {
        // TODO: unbind closures from socket.
        if self.state.get() != ClientState::Closed {
            drop(self.socket.close());
            self.state.set(ClientState::Closed);
        }
        self
    }

    fn id(&self) -> ClientID {
        self.id
    }

    fn state(&self) -> ClientState {
        self.state.get()
    }

    fn send(&mut self, id: MessageID, data: &[u8]) -> Option<Range<usize>> {
        if self.state.get() == ClientState::Open {
            let size = data.len();
            let mut stream = Cursor::new(Vec::<u8>::with_capacity(size + 8));
            drop(stream.write_u32::<BigEndian>(id.id()));
            drop(stream.write_u32::<BigEndian>(id.version()));
            drop(stream.write(data));
            let data = stream.into_inner();
            if self.socket.send_with_u8_array(&data).is_ok() {
                return Some(0..size);
            }
        }
        None
    }

    fn read(&mut self) -> Option<MsgData> {
        self.messages.borrow_mut().pop_front()
    }

    fn read_all(&mut self) -> Vec<MsgData> {
        let mut messages = self.messages.borrow_mut();
        let result = messages.iter().cloned().collect();
        messages.clear();
        result
    }
}
