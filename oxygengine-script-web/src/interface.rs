use crate::{component::WebScriptComponent, web_api::EntityId};
use core::ecs::{world::EntitiesRes, Builder, LazyUpdate};
use js_sys::{Function, JsString, Object};
use std::{collections::HashMap, sync::Mutex};
use wasm_bindgen::{JsCast, JsValue};

lazy_static! {
    static ref INTERFACE: Mutex<WebScriptInterface> = Mutex::new(WebScriptInterface::default());
}

pub struct WebScriptInterface {
    component_factory: HashMap<String, Function>,
    state_factory: HashMap<String, Function>,
    index_generator: u64,
    generation: u32,
    entities_to_create: Vec<(JsValue, EntityId)>,
}

unsafe impl Send for WebScriptInterface {}
unsafe impl Sync for WebScriptInterface {}

impl Default for WebScriptInterface {
    fn default() -> Self {
        Self {
            component_factory: HashMap::new(),
            state_factory: HashMap::new(),
            index_generator: 0,
            generation: 1,
            entities_to_create: vec![],
        }
    }
}

impl WebScriptInterface {
    pub fn register_component_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.component_factory.insert(name.to_owned(), factory);
        }
    }

    pub fn register_state_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.state_factory.insert(name.to_owned(), factory);
        }
    }

    pub fn build_state(name: &str) -> Option<JsValue> {
        if let Ok(interface) = INTERFACE.lock() {
            if let Some(factory) = interface.state_factory.get(name) {
                if let Ok(result) = factory.call0(&JsValue::UNDEFINED) {
                    return Some(result);
                }
            }
        }
        None
    }

    pub fn create_entity(data: JsValue) -> EntityId {
        if let Ok(mut interface) = INTERFACE.lock() {
            if interface.index_generator == std::u64::MAX {
                interface.index_generator = 0;
                interface.generation += 1;
            }
            let index = interface.index_generator;
            interface.index_generator += 1;
            let id = EntityId::new(index, interface.generation);
            interface.entities_to_create.push((data, id));
            id
        } else {
            EntityId::default()
        }
    }

    pub(crate) fn build_entities(entities: &EntitiesRes, lazy: &LazyUpdate) {
        if let Ok(mut interface) = INTERFACE.lock() {
            let entities_to_create = std::mem::replace(&mut interface.entities_to_create, vec![]);
            for (data, id) in entities_to_create {
                let mut builder = lazy.create_entity(entities);
                let mut components = HashMap::new();
                // TODO: i beg myself, please refactor this shit.
                if !data.is_null() && !data.is_undefined() {
                    if let Some(object) = Object::try_from(&data) {
                        let keys_iter = Object::keys(&object)
                            .iter()
                            .map(|key| key.dyn_ref::<JsString>().map(|key| String::from(key)))
                            .collect::<Vec<_>>();
                        let values_iter = Object::values(&object).iter().collect::<Vec<_>>();
                        for (key, value) in keys_iter.into_iter().zip(values_iter.into_iter()) {
                            if let Some(key) = key {
                                if let Some(factory) = interface.component_factory.get(&key) {
                                    if let Ok(v) = factory.call0(&JsValue::UNDEFINED) {
                                        if let Some(d) = Object::try_from(&v) {
                                            if let Some(o) = Object::try_from(&value) {
                                                components.insert(key, Object::assign(d, o).into());
                                            } else {
                                                components.insert(key, v);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                builder = builder.with(WebScriptComponent::new(id, components));
                builder.build();
            }
        }
    }
}
