use crate::web_api::EntityId;
use core::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WebScriptComponentState {
    Born,
    Alive,
    Killed,
}

#[derive(Debug)]
pub struct WebScriptComponent {
    id: EntityId,
    state: WebScriptComponentState,
    components: HashMap<String, JsValue>,
}

unsafe impl Send for WebScriptComponent {}
unsafe impl Sync for WebScriptComponent {}

impl WebScriptComponent {
    pub fn new(id: EntityId, components: HashMap<String, JsValue>) -> Self {
        Self {
            id,
            state: WebScriptComponentState::Born,
            components,
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn state(&self) -> WebScriptComponentState {
        self.state
    }

    pub fn components_iter(&self) -> impl Iterator<Item = (&String, &JsValue)> {
        self.components.iter()
    }

    pub fn component(&self, name: &str) -> Option<JsValue> {
        self.components.get(name).cloned()
    }
}

impl Component for WebScriptComponent {
    type Storage = VecStorage<Self>;
}
