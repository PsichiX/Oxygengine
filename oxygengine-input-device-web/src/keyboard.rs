use input::{device::InputDevice, Scalar};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

/// (key character, code)
pub type KeyCode = (char, String);

pub struct WebKeyboardInputDevice {
    element: EventTarget,
    keys: Rc<RefCell<HashSet<String>>>,
    sequence: Rc<RefCell<Vec<KeyCode>>>,
    last_sequence: Vec<KeyCode>,
}

unsafe impl Send for WebKeyboardInputDevice {}
unsafe impl Sync for WebKeyboardInputDevice {}

impl WebKeyboardInputDevice {
    pub fn new(element: EventTarget) -> Self {
        Self {
            element,
            keys: Default::default(),
            sequence: Rc::new(RefCell::new(Vec::with_capacity(128))),
            last_sequence: Vec::with_capacity(128),
        }
    }

    pub fn last_sequence(&self) -> &[KeyCode] {
        &self.last_sequence
    }
}

impl InputDevice for WebKeyboardInputDevice {
    fn name(&self) -> &str {
        "keyboard"
    }

    fn on_register(&mut self) {
        {
            let keys = self.keys.clone();
            let sequence = self.sequence.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                let code = event.code();
                let key = event.key();
                keys.borrow_mut().insert(code.clone());
                if let Some(value) = key.chars().next() {
                    if key.len() == 1 && (value.is_alphanumeric() || value.is_whitespace()) {
                        sequence.borrow_mut().push((value, code));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let keys = self.keys.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                keys.borrow_mut().remove(&event.code());
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }

    fn on_unregister(&mut self) {
        // TODO: cache callbacks, remove events and kill callbacks here.
    }

    fn process(&mut self) {
        self.last_sequence = self.sequence.borrow_mut().drain(..).collect();
    }

    fn query_axis(&self, name: &str) -> Option<Scalar> {
        Some(if self.keys.borrow().contains(name) {
            1.0
        } else {
            0.0
        })
    }

    fn query_trigger(&self, name: &str) -> Option<bool> {
        Some(self.keys.borrow().contains(name))
    }
}
