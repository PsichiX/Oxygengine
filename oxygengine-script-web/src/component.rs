use crate::web_api::EntityId;
use core::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub(crate) struct WebScriptComponent {
    id: EntityId,
    components: HashMap<String, Option<JsValue>>,
}

unsafe impl Send for WebScriptComponent {}
unsafe impl Sync for WebScriptComponent {}

impl WebScriptComponent {
    pub fn new(id: EntityId, components: HashMap<String, Option<JsValue>>) -> Self {
        Self { id, components }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    pub fn get_component(&self, name: &str) -> Option<JsValue> {
        self.components.get(name)?.clone()
    }

    pub fn set_component(&mut self, name: &str, value: JsValue) -> bool {
        if let Some(component) = self.components.get_mut(name) {
            if let Some(component) = component {
                *component = value;
                return true;
            }
        }
        false
    }
}

impl Component for WebScriptComponent {
    type Storage = VecStorage<Self>;
}
