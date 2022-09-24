use crate::input::device::InputDevice;
use backend::closure::WebClosure;
use core::Scalar;
use std::{
    any::Any,
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub struct WebTouchPointer {
    pub x: Scalar,
    pub y: Scalar,
    pub force: Scalar,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
    pub angle: Scalar,
}

pub struct WebTouchInputDevice {
    element: EventTarget,
    pointers: Rc<RefCell<HashMap<i32, WebTouchPointer>>>,
    started: HashSet<i32>,
    ended: HashSet<i32>,
    cached: HashSet<i32>,
    touch_start_closure: WebClosure,
    touch_end_closure: WebClosure,
    touch_move_closure: WebClosure,
    touch_cancel_closure: WebClosure,
}

unsafe impl Send for WebTouchInputDevice {}
unsafe impl Sync for WebTouchInputDevice {}

impl WebTouchInputDevice {
    pub fn new(element: EventTarget) -> Self {
        Self {
            element,
            pointers: Default::default(),
            cached: Default::default(),
            started: Default::default(),
            ended: Default::default(),
            touch_start_closure: Default::default(),
            touch_end_closure: Default::default(),
            touch_move_closure: Default::default(),
            touch_cancel_closure: Default::default(),
        }
    }

    pub fn touches(&self) -> Ref<HashMap<i32, WebTouchPointer>> {
        self.pointers.borrow()
    }

    pub fn active(&self) -> impl Iterator<Item = i32> + '_ {
        self.cached.iter().copied()
    }

    pub fn started(&self) -> impl Iterator<Item = i32> + '_ {
        self.started.iter().copied()
    }

    pub fn ended(&self) -> impl Iterator<Item = i32> + '_ {
        self.ended.iter().copied()
    }
}

impl InputDevice for WebTouchInputDevice {
    fn name(&self) -> &str {
        "touch"
    }

    fn on_register(&mut self) {
        macro_rules! impl_callback {
            ($name:expr) => {{
                let pointers = self.pointers.clone();
                let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
                    let list = event.target_touches();
                    let count = list.length();
                    let mut pointers = pointers.borrow_mut();
                    pointers.clear();
                    pointers.reserve(count as _);
                    for i in 0..count {
                        let item = list.item(i).unwrap();
                        pointers.insert(
                            item.identifier(),
                            WebTouchPointer {
                                x: item.client_x() as _,
                                y: item.client_y() as _,
                                force: item.force() as _,
                                radius_x: item.radius_x() as _,
                                radius_y: item.radius_y() as _,
                                angle: item.rotation_angle() as _,
                            },
                        );
                    }
                }) as Box<dyn FnMut(_)>);
                self.element
                    .add_event_listener_with_callback($name, closure.as_ref().unchecked_ref())
                    .unwrap();
                WebClosure::acquire(closure)
            }};
        }

        self.touch_start_closure = impl_callback!("touchstart");
        self.touch_end_closure = impl_callback!("touchend");
        self.touch_move_closure = impl_callback!("touchmove");
        self.touch_cancel_closure = impl_callback!("touchcancel");
    }

    fn on_unregister(&mut self) {
        self.touch_start_closure.release();
        self.touch_end_closure.release();
        self.touch_move_closure.release();
        self.touch_cancel_closure.release();
    }

    fn process(&mut self) {
        let pointers = self.pointers.borrow();
        self.started.clear();
        self.started.reserve(pointers.len());
        for id in pointers.keys() {
            if self.cached.contains(id) {
                self.started.insert(*id);
            }
        }
        self.ended.clear();
        self.ended.reserve(pointers.len());
        for id in &self.cached {
            if !pointers.contains_key(id) {
                self.ended.insert(*id);
            }
        }
        self.cached.clear();
        self.cached.reserve(pointers.len());
        self.cached.extend(pointers.keys().copied());
    }

    fn query_axis(&self, name: &str) -> Option<Scalar> {
        match name {
            "x" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.x),
            "y" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.y),
            "force" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.force),
            "radius-x" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.radius_x),
            "radius-y" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.radius_y),
            "angle" => self
                .pointers
                .borrow()
                .values()
                .next()
                .map(|pointer| pointer.angle),
            _ => None,
        }
    }

    fn query_trigger(&self, name: &str) -> Option<bool> {
        match name {
            "touch" => Some(!self.pointers.borrow().is_empty()),
            _ => None,
        }
    }

    fn query_text(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
