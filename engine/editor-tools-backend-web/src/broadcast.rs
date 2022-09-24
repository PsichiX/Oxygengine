use js_sys::{Object, Uint8ClampedArray};
use oxygengine_backend_web::closure::WebClosure;
use oxygengine_editor_tools::simp::*;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{BroadcastChannel, MessageEvent};

pub struct WebBroadcastChannel {
    name: String,
    channel: BroadcastChannel,
    callback: WebClosure,
    messages: Rc<RefCell<VecDeque<SimpMessage>>>,
}

#[cfg(feature = "web")]
unsafe impl Send for WebBroadcastChannel {}
#[cfg(feature = "web")]
unsafe impl Sync for WebBroadcastChannel {}

impl WebBroadcastChannel {
    pub fn new(name: impl ToString) -> Self {
        let name = name.to_string();
        let channel = BroadcastChannel::new(&name).unwrap();
        let messages = Rc::new(RefCell::new(VecDeque::default()));
        let messages2 = Rc::clone(&messages);
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
            event.prevent_default();
            let id = js_sys::Reflect::get(&event.data(), &JsValue::from("id"))
                .unwrap()
                .as_string()
                .unwrap();
            let version = js_sys::Reflect::get(&event.data(), &JsValue::from("version"))
                .unwrap()
                .as_f64()
                .unwrap() as u32;
            let text_data = js_sys::Reflect::get(&event.data(), &JsValue::from("text"))
                .ok()
                .and_then(|text| text.as_string());
            let binary_data = js_sys::Reflect::get(&event.data(), &JsValue::from("binary"))
                .ok()
                .and_then(|binary| binary.dyn_into::<Uint8ClampedArray>().ok())
                .map(|binary| binary.to_vec());
            messages2.borrow_mut().push_back(SimpMessage {
                id: SimpMessageId::new(id, version),
                text_data,
                binary_data,
            });
        }) as Box<dyn FnMut(_)>);
        channel
            .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
            .unwrap();
        Self {
            name,
            channel,
            callback: WebClosure::acquire(closure),
            messages,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for WebBroadcastChannel {
    fn drop(&mut self) {
        self.channel.close();
        self.callback.release();
    }
}

impl SimpSender for WebBroadcastChannel {
    type Error = ();

    fn write(&mut self, message: SimpMessage) -> Result<(), Self::Error> {
        let SimpMessage {
            id,
            text_data,
            binary_data,
        } = message;
        let SimpMessageId { id, version } = id;
        let result = Object::new();
        js_sys::Reflect::set(&result, &JsValue::from("id"), &JsValue::from(id)).unwrap();
        js_sys::Reflect::set(
            &result,
            &JsValue::from("version"),
            &JsValue::from(version as f64),
        )
        .unwrap();
        if let Some(text_data) = text_data {
            js_sys::Reflect::set(&result, &JsValue::from("text"), &JsValue::from(text_data))
                .unwrap();
        }
        if let Some(binary_data) = binary_data {
            let buffer = Uint8ClampedArray::new_with_length(binary_data.len() as _);
            buffer.copy_from(&binary_data);
            js_sys::Reflect::set(&result, &JsValue::from("binary"), &buffer).unwrap();
        }
        self.channel.post_message(&result).map_err(|_| ())
    }
}

impl SimpReceiver for WebBroadcastChannel {
    type Error = ();

    fn read(&mut self) -> Option<Result<SimpMessage, Self::Error>> {
        self.messages.borrow_mut().pop_front().map(Ok)
    }
}

impl SimpChannel for WebBroadcastChannel {}
