use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use js_sys::{Array, Function, JsString};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub(crate) index: u64,
    pub(crate) generation: u32,
}

#[wasm_bindgen]
impl EntityId {
    #[wasm_bindgen(constructor)]
    pub fn new_invalid() -> Result<(), JsValue> {
        Err(JsValue::from_str(
            "Tried to create EntityId from constructor!",
        ))
    }

    pub(crate) fn new(index: u64, generation: u32) -> Self {
        Self { index, generation }
    }

    pub fn is_valid(&self) -> bool {
        self.generation > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Constrain {
    None,
    Entities,
    Resource(String),
    Components(String),
    ExcludeComponents(String),
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebScriptFetch {
    index: usize,
    constrains: Vec<Constrain>,
}

impl WebScriptFetch {
    pub(crate) fn new(constrains: Vec<Constrain>) -> Self {
        Self {
            index: 0,
            constrains,
        }
    }
}

#[wasm_bindgen]
impl WebScriptFetch {
    #[wasm_bindgen(constructor)]
    pub fn new_invalid() -> Result<(), JsValue> {
        Err(JsValue::from_str("Tried to create WebScriptFetch from constructor! Use WebScriptApi::fetch() to create valid fetch object."))
    }

    // TODO: refactor this shit, please.
    pub fn next(&mut self) -> bool {
        if let Some(world) = WebScriptInterface::world() {
            if let Some(world) = unsafe { world.as_mut() } {
                'main: while let Some(entity) = WebScriptInterface::get_entity(self.index) {
                    self.index += 1;
                    if let Some(c) = world.read_storage::<WebScriptComponent>().get(entity) {
                        for constrain in &self.constrains {
                            match constrain {
                                Constrain::Entities => {}
                                Constrain::Resource(name) => {
                                    if !WebScriptInterface::has_resource(name) {
                                        continue 'main;
                                    }
                                }
                                Constrain::Components(name) => {
                                    if !c.has_component(name) {
                                        continue 'main;
                                    }
                                }
                                Constrain::ExcludeComponents(name) => {
                                    if c.has_component(name) {
                                        continue 'main;
                                    }
                                }
                                Constrain::None => {
                                    continue 'main;
                                }
                            };
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    // TODO: refactor this shit, please.
    pub fn read(&mut self, item: usize) -> JsValue {
        if self.index == 0 {
            return JsValue::UNDEFINED;
        }
        if let Some(constrain) = self.constrains.get(item) {
            let index = self.index - 1;
            if let Some(entity) = WebScriptInterface::get_entity(index) {
                if let Some(world) = WebScriptInterface::world() {
                    if let Some(world) = unsafe { world.as_ref() } {
                        if let Some(c) = world.read_storage::<WebScriptComponent>().get(entity) {
                            match constrain {
                                Constrain::Entities => return c.id().into(),
                                Constrain::Resource(name) => {
                                    if let Some(resource) = WebScriptInterface::get_resource(name) {
                                        return resource;
                                    }
                                }
                                Constrain::Components(name) => {
                                    if let Some(component) = c.get_component(name) {
                                        return component;
                                    } else if let Some(component) =
                                        WebScriptInterface::read_component_bridge(name, entity)
                                    {
                                        return component;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        JsValue::UNDEFINED
    }

    // TODO: refactor this shit, please.
    pub fn write(&mut self, item: usize, value: JsValue) {
        if self.index == 0 {
            return;
        }
        if let Some(constrain) = self.constrains.get(item) {
            let index = self.index - 1;
            if let Some(entity) = WebScriptInterface::get_entity(index) {
                if let Some(world) = WebScriptInterface::world() {
                    if let Some(world) = unsafe { world.as_mut() } {
                        if let Some(c) = world.write_storage::<WebScriptComponent>().get_mut(entity)
                        {
                            match constrain {
                                Constrain::Resource(name) => {
                                    WebScriptInterface::set_resource(name, value);
                                }
                                Constrain::Components(name) => {
                                    if !c.set_component(name, value.clone()) {
                                        WebScriptInterface::write_component_bridge(
                                            name, entity, value,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct WebScriptApi;

#[wasm_bindgen]
impl WebScriptApi {
    #[wasm_bindgen(js_name = "start")]
    pub fn start() {
        WebScriptInterface::start();
    }

    #[wasm_bindgen(js_name = "registerResource")]
    pub fn register_resource(name: &str, resource: JsValue) {
        WebScriptInterface::register_resource(name, resource);
    }

    #[wasm_bindgen(js_name = "registerStateFactory")]
    pub fn register_state_factory(name: &str, factory: Function) {
        WebScriptInterface::register_state_factory(name, factory);
    }

    #[wasm_bindgen(js_name = "registerComponentFactory")]
    pub fn register_component_factory(name: &str, factory: Function) {
        WebScriptInterface::register_component_factory(name, factory);
    }

    #[wasm_bindgen(js_name = "registerSystem")]
    pub fn register_system(name: &str, system: JsValue) {
        WebScriptInterface::register_system(name, system);
    }

    #[wasm_bindgen(js_name = "createEntity")]
    pub fn create_entity(data: JsValue) -> EntityId {
        WebScriptInterface::create_entity(data)
    }

    #[wasm_bindgen(js_name = "destroyEntity")]
    pub fn destroy_entity(id: EntityId) {
        WebScriptInterface::destroy_entity(id);
    }

    pub fn fetch(constrains: &Array) -> Option<WebScriptFetch> {
        let constrains = constrains
            .iter()
            .map(|c| {
                if let Some(s) = c.dyn_ref::<JsString>() {
                    let s = String::from(s);
                    if s.starts_with("@") {
                        return Constrain::Entities;
                    } else if s.starts_with("$") {
                        return Constrain::Resource(s[1..].to_owned());
                    } else if s.starts_with("+") {
                        return Constrain::Components(s[1..].to_owned());
                    } else if s.starts_with("-") {
                        return Constrain::ExcludeComponents(s[1..].to_owned());
                    }
                }
                Constrain::None
            })
            .collect::<Vec<_>>();
        Some(WebScriptFetch::new(constrains))
    }
}
