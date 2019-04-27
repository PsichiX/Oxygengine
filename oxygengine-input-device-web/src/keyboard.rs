use input::{device::InputDevice, Scalar};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub struct WebKeyboardInputDevice {
    element: EventTarget,
    keys: Rc<RefCell<HashSet<String>>>,
}

unsafe impl Send for WebKeyboardInputDevice {}
unsafe impl Sync for WebKeyboardInputDevice {}

impl WebKeyboardInputDevice {
    pub fn new(element: EventTarget) -> Self {
        Self {
            element,
            keys: Default::default(),
        }
    }
}

impl InputDevice for WebKeyboardInputDevice {
    fn name(&self) -> &str {
        "keyboard"
    }

    fn on_register(&mut self) {
        {
            let keys = self.keys.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                keys.borrow_mut().insert(event.code());
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

    fn process(&mut self) {}

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
