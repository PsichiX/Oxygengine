use crate::interface::WebScriptInterface;
use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct EntityId {
    pub(crate) index: u64,
    pub(crate) generation: u32,
}

#[wasm_bindgen]
impl EntityId {
    pub(crate) fn new(index: u64, generation: u32) -> Self {
        Self { index, generation }
    }

    pub fn is_valid(&self) -> bool {
        self.generation > 0
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct WebScriptApi;

#[wasm_bindgen]
impl WebScriptApi {
    pub fn register_state_factory(name: &str, factory: Function) {
        WebScriptInterface::register_state_factory(name, factory);
    }

    pub fn register_component_factory(name: &str, factory: Function) {
        WebScriptInterface::register_component_factory(name, factory);
    }

    pub fn create_entity(data: JsValue) -> EntityId {
        WebScriptInterface::create_entity(data)
    }
}
