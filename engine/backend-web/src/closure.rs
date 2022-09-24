use std::any::Any;
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
pub struct WebClosure(Option<Box<dyn Any>>);

impl WebClosure {
    pub fn acquire<T: ?Sized + 'static>(closure: Closure<T>) -> Self {
        Self(Some(Box::new(closure)))
    }

    pub fn release(&mut self) {
        self.0 = None;
    }
}

#[cfg(feature = "web")]
unsafe impl Send for WebClosure {}
#[cfg(feature = "web")]
unsafe impl Sync for WebClosure {}
