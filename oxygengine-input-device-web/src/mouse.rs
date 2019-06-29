use input::{device::InputDevice, Scalar};
use std::{any::Any, cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub struct WebMouseInputDevice {
    element: EventTarget,
    position: Rc<Cell<(Scalar, Scalar)>>,
    left_button: Rc<Cell<bool>>,
    right_button: Rc<Cell<bool>>,
    middle_button: Rc<Cell<bool>>,
}

unsafe impl Send for WebMouseInputDevice {}
unsafe impl Sync for WebMouseInputDevice {}

impl WebMouseInputDevice {
    pub fn new(element: EventTarget) -> Self {
        Self {
            element,
            position: Default::default(),
            left_button: Default::default(),
            right_button: Default::default(),
            middle_button: Default::default(),
        }
    }
}

impl InputDevice for WebMouseInputDevice {
    fn name(&self) -> &str {
        "mouse"
    }

    fn on_register(&mut self) {
        {
            let left_button = self.left_button.clone();
            let right_button = self.right_button.clone();
            let middle_button = self.middle_button.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| match event.button() {
                0 => left_button.set(true),
                2 => right_button.set(true),
                1 => middle_button.set(true),
                _ => {}
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let left_button = self.left_button.clone();
            let right_button = self.right_button.clone();
            let middle_button = self.middle_button.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| match event.button() {
                0 => left_button.set(false),
                2 => right_button.set(false),
                1 => middle_button.set(false),
                _ => {}
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let position = self.position.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                position.set((event.client_x() as Scalar, event.client_y() as Scalar));
            }) as Box<dyn FnMut(_)>);
            self.element
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }

    fn on_unregister(&mut self) {
        // TODO: cache callbacks, remove events and kill callbacks here.
    }

    fn process(&mut self) {}

    fn query_axis(&self, name: &str) -> Option<Scalar> {
        match name {
            "x" => Some(self.position.get().0),
            "y" => Some(self.position.get().1),
            _ => None,
        }
    }

    fn query_trigger(&self, name: &str) -> Option<bool> {
        match name {
            "left" => Some(self.left_button.get()),
            "right" => Some(self.right_button.get()),
            "middle" => Some(self.middle_button.get()),
            _ => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
