use core::prelude::*;
use js_sys::Object;
use std::collections::HashMap;

#[derive(Default)]
pub struct WebScriptComponent(pub HashMap<String, Object>);

unsafe impl Send for WebScriptComponent {}
unsafe impl Sync for WebScriptComponent {}

impl WebScriptComponent {
    // pub fn new() {}
}

impl Component for WebScriptComponent {
    type Storage = VecStorage<Self>;
}
