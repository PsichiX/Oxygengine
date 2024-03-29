use backend::closure::WebClosure;
use core::{ecs::Universe, Scalar};
use input::device::InputDevice;
use std::{any::Any, cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

/// (key character, code)
pub type KeyCode = (char, String);

pub struct WebKeyboardInputDevice {
    element: EventTarget,
    keys: Rc<RefCell<HashSet<String>>>,
    sequence: Rc<RefCell<Vec<KeyCode>>>,
    last_sequence: Vec<KeyCode>,
    key_down_closure: WebClosure,
    key_up_closure: WebClosure,
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
            key_down_closure: Default::default(),
            key_up_closure: Default::default(),
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
                match key.len() {
                    1 => sequence
                        .borrow_mut()
                        .push((key.chars().next().unwrap(), code)),
                    2..=std::usize::MAX => sequence.borrow_mut().push((0 as char, code)),
                    _ => {}
                }
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();
            self.key_down_closure = WebClosure::acquire(closure);
        }
        {
            let keys = self.keys.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                keys.borrow_mut().remove(&event.code());
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
                .unwrap();
            self.key_up_closure = WebClosure::acquire(closure);
        }
    }

    fn on_unregister(&mut self) {
        self.key_down_closure.release();
        self.key_up_closure.release();
    }

    fn process(&mut self, _: &mut Universe) {
        self.last_sequence.clear();
        self.last_sequence.append(&mut self.sequence.borrow_mut());
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

    fn query_text(&self) -> Option<String> {
        Some(self.last_sequence.iter().map(|(c, _)| c).collect())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
