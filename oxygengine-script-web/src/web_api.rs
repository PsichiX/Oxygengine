use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use js_sys::{Array, Function, JsString};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub(crate) enum Constrain {
    None,
    Entities,
    Resource(String),
    Components(String),
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebScriptFetch {
    index: usize,
    constrains: Vec<Constrain>,
    cached_item: Array,
}

impl WebScriptFetch {
    pub(crate) fn new(constrains: Vec<Constrain>) -> Self {
        let cached_item = Array::new_with_length(constrains.len() as u32);
        Self {
            index: 0,
            constrains,
            cached_item,
        }
    }
}

#[wasm_bindgen]
impl WebScriptFetch {
    #[wasm_bindgen(constructor)]
    pub fn new_invalid() -> Result<JsValue, JsValue> {
        Err(JsValue::from_str("Tried to create WebScriptFetch from constructor! Use WebScriptApi::fetch() to create valid fetch object."))
    }

    // TODO: refactor this shit, please.
    pub fn next(&mut self) -> bool {
        if let Some(world) = WebScriptInterface::world() {
            if let Some(world) = unsafe { world.as_mut() } {
                'main: while let Some(entity) = WebScriptInterface::get_entity(self.index) {
                    self.index += 1;
                    if let Some(c) = world.read_storage::<WebScriptComponent>().get(entity) {
                        for (i, constrain) in self.constrains.iter().enumerate() {
                            let value = match constrain {
                                Constrain::Entities => c.id().into(),
                                Constrain::Resource(name) => {
                                    if let Some(r) = WebScriptInterface::get_resource(name) {
                                        r
                                    } else {
                                        continue 'main;
                                    }
                                }
                                Constrain::Components(name) => {
                                    if let Some(c) = c.component(name) {
                                        c
                                    } else {
                                        continue 'main;
                                    }
                                }
                                _ => {
                                    continue 'main;
                                }
                            };
                            self.cached_item.set(i as u32, value);
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn current(&self) -> JsValue {
        let r: &JsValue = self.cached_item.as_ref();
        r.clone()
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct WebScriptApi;

#[wasm_bindgen]
impl WebScriptApi {
    #[wasm_bindgen(js_name = "markReady")]
    pub fn mark_ready() {
        WebScriptInterface::mark_ready();
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
                    } else if s.starts_with("&") {
                        return Constrain::Components(s[1..].to_owned());
                    }
                }
                Constrain::None
            })
            .collect::<Vec<_>>();
        Some(WebScriptFetch::new(constrains))
    }
}
