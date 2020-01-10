use crate::{component::WebScriptComponent, web_api::EntityId};
use core::ecs::{Builder, Entity, World};
use js_sys::{Function, JsString, Object, Reflect};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use wasm_bindgen::{JsCast, JsValue};

lazy_static! {
    static ref INTERFACE: Mutex<WebScriptInterface> = Mutex::new(WebScriptInterface::default());
}

pub struct WebScriptInterface {
    ready: bool,
    world_ptr: Option<*mut World>,
    resources: HashMap<String, JsValue>,
    component_factory: HashMap<String, Function>,
    state_factory: HashMap<String, Function>,
    systems: HashMap<String, (JsValue, Function)>,
    index_generator: u64,
    generation: u32,
    entities_to_create: Vec<(JsValue, EntityId)>,
    entities_to_destroy: HashSet<EntityId>,
    entities_map: HashMap<EntityId, Entity>,
}

unsafe impl Send for WebScriptInterface {}
unsafe impl Sync for WebScriptInterface {}

impl Default for WebScriptInterface {
    fn default() -> Self {
        Self {
            ready: false,
            world_ptr: None,
            resources: HashMap::new(),
            component_factory: HashMap::new(),
            state_factory: HashMap::new(),
            systems: HashMap::new(),
            index_generator: 0,
            generation: 1,
            entities_to_create: vec![],
            entities_to_destroy: HashSet::new(),
            entities_map: HashMap::new(),
        }
    }
}

impl WebScriptInterface {
    pub fn is_ready() -> bool {
        if let Ok(interface) = INTERFACE.lock() {
            interface.ready
        } else {
            false
        }
    }

    pub(crate) fn mark_ready() {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.ready = true;
        }
    }

    pub fn is_invalid() -> bool {
        if let Ok(interface) = INTERFACE.lock() {
            interface.world_ptr.is_none()
        } else {
            true
        }
    }

    pub(crate) fn set_world(world: &mut World) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.world_ptr = Some(world as *mut World);
        }
    }

    pub(crate) fn unset_world() {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.world_ptr = None;
        }
    }

    pub(crate) fn world() -> Option<*mut World> {
        if let Ok(interface) = INTERFACE.lock() {
            interface.world_ptr
        } else {
            None
        }
    }

    pub(crate) fn entities() -> Option<Vec<Entity>> {
        if let Ok(interface) = INTERFACE.lock() {
            Some(interface.entities_map.values().copied().collect::<Vec<_>>())
        } else {
            None
        }
    }

    pub fn register_resource(name: &str, resource: JsValue) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.resources.insert(name.to_owned(), resource);
            }
        }
    }

    pub fn register_component_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.component_factory.insert(name.to_owned(), factory);
            }
        }
    }

    pub fn register_state_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.state_factory.insert(name.to_owned(), factory);
            }
        }
    }

    pub fn register_system(name: &str, system: JsValue) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                if let Ok(m) = Reflect::get(&system, &JsValue::from_str("onRun")) {
                    if let Some(m) = m.dyn_ref::<Function>() {
                        interface
                            .systems
                            .insert(name.to_owned(), (system, m.clone()));
                    }
                }
            }
        }
    }

    pub fn get_resource(name: &str) -> JsValue {
        if let Ok(interface) = INTERFACE.lock() {
            if let Some(resource) = interface.resources.get(name) {
                return resource.clone();
            }
        }
        JsValue::UNDEFINED
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

    pub fn destroy_entity(id: EntityId) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.entities_to_destroy.insert(id);
        }
    }

    pub(crate) fn run_systems() {
        // TODO: figure out how to avoid allocation (allocation is made to unlock access to
        // interface from JS systems).
        let meta = if let Ok(interface) = INTERFACE.lock() {
            interface.systems.values().cloned().collect::<Vec<_>>()
        } else {
            return;
        };
        for (context, on_run) in meta {
            drop(on_run.call0(&context));
        }
    }

    pub(crate) fn maintain_entities(world: &mut World) {
        if let Ok(mut interface) = INTERFACE.lock() {
            let entities_to_destroy =
                std::mem::replace(&mut interface.entities_to_destroy, HashSet::new());
            for id in entities_to_destroy {
                if let Some(id) = interface.entities_map.remove(&id) {
                    drop(world.delete_entity(id));
                }
            }

            let entities_to_create = std::mem::replace(&mut interface.entities_to_create, vec![]);
            for (data, id) in entities_to_create {
                let mut builder = world.create_entity();
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
                interface.entities_map.insert(id, builder.build());
            }
        }
    }
}
