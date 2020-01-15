use crate::{
    component::WebScriptComponent,
    scriptable::{
        scriptable_js_to_value, scriptable_value_merge, scriptable_value_to_js, Scriptable,
        ScriptableValue,
    },
    state::WebScriptStateScripted,
    web_api::EntityId,
};
use core::ecs::{Builder, Component, Entity, EntityBuilder, Resource, World};
use js_sys::{Function, JsString, Object, Reflect};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use wasm_bindgen::{JsCast, JsValue};

lazy_static! {
    static ref INTERFACE: Mutex<WebScriptInterface> = Mutex::new(WebScriptInterface::default());
}

pub trait ComponentModify<R> {
    fn modify_component(&mut self, source: R);
}

impl<T, R> ComponentModify<R> for T
where
    T: Component + From<R>,
{
    fn modify_component(&mut self, source: R) {
        *self = source.into();
    }
}

struct ComponentBridge {
    add_to_entity: Box<dyn FnMut(EntityBuilder, JsValue) -> EntityBuilder>,
    read_data: Box<dyn FnMut(&World, Entity) -> JsValue>,
    write_data: Box<dyn FnMut(&mut World, Entity, JsValue)>,
}

impl ComponentBridge {
    fn on_add_to_entity<'a>(
        &mut self,
        data: JsValue,
        builder: EntityBuilder<'a>,
    ) -> EntityBuilder<'a> {
        (self.add_to_entity)(builder, data)
    }

    fn on_read_data(&mut self, world: &World, entity: Entity) -> JsValue {
        (self.read_data)(world, entity)
    }

    fn on_write_data(&mut self, world: &mut World, entity: Entity, value: JsValue) {
        (self.write_data)(world, entity, value)
    }
}

pub trait ResourceModify<R> {
    fn modify_resource(&mut self, source: R);
}

struct ResourceBridge {
    read_data: Box<dyn FnMut(&World) -> JsValue>,
    write_data: Box<dyn FnMut(&mut World, JsValue)>,
}

impl ResourceBridge {
    fn on_read_data(&mut self, world: &World) -> JsValue {
        (self.read_data)(world)
    }

    fn on_write_data(&mut self, world: &mut World, value: JsValue) {
        (self.write_data)(world, value)
    }
}

pub struct WebScriptInterface {
    ready: bool,
    // TODO: check if this pointer can be pinned.
    world_ptr: Option<*mut World>,
    resources: HashMap<String, JsValue>,
    resources_bridge: HashMap<String, ResourceBridge>,
    component_factory: HashMap<String, Function>,
    scriptable_components: HashMap<String, Box<dyn Scriptable>>,
    components_bridge: HashMap<String, ComponentBridge>,
    state_factory: HashMap<String, Function>,
    scriptable_state_factory: HashMap<String, Box<dyn FnMut() -> Box<dyn WebScriptStateScripted>>>,
    systems: HashMap<String, (JsValue, Function)>,
    systems_cache: Option<Vec<(JsValue, Function)>>,
    index_generator: u64,
    generation: u32,
    entities_to_create: Vec<(JsValue, EntityId)>,
    entities_to_destroy: HashSet<EntityId>,
    entities_map: HashMap<EntityId, Entity>,
    entities_cache: Vec<Entity>,
}

unsafe impl Send for WebScriptInterface {}
unsafe impl Sync for WebScriptInterface {}

impl Default for WebScriptInterface {
    fn default() -> Self {
        Self {
            ready: false,
            world_ptr: None,
            resources: HashMap::new(),
            resources_bridge: HashMap::new(),
            component_factory: HashMap::new(),
            scriptable_components: HashMap::new(),
            components_bridge: HashMap::new(),
            state_factory: HashMap::new(),
            scriptable_state_factory: HashMap::new(),
            systems: HashMap::new(),
            systems_cache: None,
            index_generator: 0,
            generation: 1,
            entities_to_create: vec![],
            entities_to_destroy: HashSet::new(),
            entities_map: HashMap::new(),
            entities_cache: vec![],
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

    pub fn is_invalid() -> bool {
        if let Ok(interface) = INTERFACE.lock() {
            interface.world_ptr.is_none()
        } else {
            true
        }
    }

    pub fn register_scriptable_resource<S>(&mut self, name: &str, resource: S)
    where
        S: 'static + Scriptable,
    {
        if !self.ready {
            if let Ok(resource) = resource.to_js() {
                self.resources.insert(name.to_owned(), resource);
            }
        }
    }

    pub fn register_resource_bridge<'a, S, M>(&mut self, name: &str)
    where
        S: Resource + Send + Sync + ResourceModify<M>,
        M: Scriptable + From<&'a S>,
    {
        self.resources_bridge.insert(
            name.to_owned(),
            ResourceBridge {
                read_data: Box::new(|world| {
                    let r: &S = &world.read_resource::<S>();
                    // TODO: this is very hacky and extends reference lifetime for that time that
                    // resource is converted into proxy object.
                    // CHANGE IT IN THE FUTURE.
                    let data = M::from(unsafe { std::mem::transmute(r) });
                    if let Ok(data) = data.to_js() {
                        data
                    } else {
                        JsValue::UNDEFINED
                    }
                }),
                write_data: Box::new(|world, value| {
                    if let Ok(data) = M::from_js(value) {
                        let r: &mut S = &mut world.write_resource::<S>();
                        r.modify_resource(data);
                    }
                }),
            },
        );
    }

    pub fn register_scriptable_component<S>(&mut self, name: &str, component: S)
    where
        S: 'static + Scriptable,
    {
        if !self.ready {
            self.scriptable_components
                .insert(name.to_owned(), Box::new(component));
        }
    }

    pub fn register_component_bridge<S, M>(&mut self, name: &str, template: S)
    where
        S: Scriptable + Component + Clone + Send + Sync + ComponentModify<M>,
        S::Storage: Default,
        M: Scriptable + From<S>,
    {
        self.components_bridge.insert(
            name.to_owned(),
            ComponentBridge {
                add_to_entity: Box::new(move |builder, data| {
                    let template_medium: M = template.clone().into();
                    if let Ok(template_scriptable) = template_medium.to_scriptable() {
                        if let Ok(data_scriptable) = scriptable_js_to_value(data) {
                            let merged_scriptable =
                                scriptable_value_merge(&template_scriptable, &data_scriptable);
                            if let Ok(data) = M::from_scriptable(&merged_scriptable) {
                                let mut template = template.clone();
                                template.modify_component(data);
                                return builder.with(template);
                            }
                        }
                    }
                    builder
                }),
                read_data: Box::new(|world, entity| {
                    if let Some(data) = world.read_storage::<S>().get(entity) {
                        let data: M = data.clone().into();
                        if let Ok(data) = data.to_js() {
                            return data;
                        }
                    }
                    JsValue::UNDEFINED
                }),
                write_data: Box::new(|world, entity, value| {
                    if let Ok(data) = M::from_js(value) {
                        if let Some(component) = world.write_storage::<S>().get_mut(entity) {
                            component.modify_component(data);
                        }
                    }
                }),
            },
        );
    }

    pub fn register_scriptable_state_factory<S>(&mut self, name: &str, factory: S)
    where
        S: 'static + FnMut() -> Box<dyn WebScriptStateScripted>,
    {
        if !self.ready {
            self.scriptable_state_factory
                .insert(name.to_owned(), Box::new(factory));
        }
    }

    pub fn read_scriptable_resource<T>(name: &str) -> Option<T>
    where
        T: Scriptable,
    {
        if let Some(resource) = Self::get_resource(name) {
            if let Ok(resource) = T::from_js(resource) {
                return Some(resource);
            }
        }
        None
    }

    pub fn read_js_resource(name: &str) -> Option<ScriptableValue> {
        if let Some(resource) = Self::get_resource(name) {
            if let Ok(resource) = scriptable_js_to_value(resource) {
                return Some(resource);
            }
        }
        None
    }

    pub fn write_scriptable_resource<T>(name: &str, value: &T)
    where
        T: Scriptable,
    {
        if let Ok(resource) = value.to_js() {
            Self::set_resource(name, resource);
        }
    }

    pub fn write_js_resource(name: &str, value: &ScriptableValue) {
        if let Ok(resource) = scriptable_value_to_js(value) {
            Self::set_resource(name, resource);
        }
    }

    pub(crate) fn register_resource(name: &str, resource: JsValue) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.resources.insert(name.to_owned(), resource);
            }
        }
    }

    pub(crate) fn register_component_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.component_factory.insert(name.to_owned(), factory);
            }
        }
    }

    pub(crate) fn register_system(name: &str, system: JsValue) {
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

    pub(crate) fn register_state_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready {
                interface.state_factory.insert(name.to_owned(), factory);
            }
        }
    }

    pub(crate) fn with<F, R>(mut f: F) -> R
    where
        F: FnMut(&mut Self) -> R,
        R: Default,
    {
        if let Ok(mut interface) = INTERFACE.lock() {
            f(&mut interface)
        } else {
            R::default()
        }
    }

    pub(crate) fn start() {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.ready = true;
            interface.systems_cache = Some(interface.systems.values().cloned().collect::<Vec<_>>());
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

    pub(crate) fn get_entity(index: usize) -> Option<Entity> {
        if let Ok(interface) = INTERFACE.lock() {
            interface.entities_cache.get(index).copied()
        } else {
            None
        }
    }

    pub(crate) fn get_resource(name: &str) -> Option<JsValue> {
        if let Ok(interface) = INTERFACE.lock() {
            if let Some(resource) = interface.resources.get(name) {
                return Some(resource.clone());
            }
        }
        None
    }

    pub(crate) fn set_resource(name: &str, value: JsValue) -> bool {
        if let Ok(mut interface) = INTERFACE.lock() {
            if interface.ready && interface.resources.contains_key(name) {
                interface.resources.insert(name.to_owned(), value);
                return true;
            }
        }
        false
    }

    pub(crate) fn read_resource_bridge(name: &str) -> Option<JsValue> {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready || interface.world_ptr.is_none() {
                return None;
            }
            let world = interface.world_ptr.unwrap();
            if let Some(bridge) = interface.resources_bridge.get_mut(name) {
                return Some(bridge.on_read_data(unsafe { world.as_ref().unwrap() }));
            }
        }
        None
    }

    pub(crate) fn write_resource_bridge(name: &str, value: JsValue) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if !interface.ready || interface.world_ptr.is_none() {
                return;
            }
            let world = interface.world_ptr.unwrap();
            if let Some(bridge) = interface.resources_bridge.get_mut(name) {
                bridge.on_write_data(unsafe { world.as_mut().unwrap() }, value);
            }
        }
    }

    pub(crate) fn read_component_bridge(name: &str, entity: Entity) -> Option<JsValue> {
        if let Ok(mut interface) = INTERFACE.lock() {
            if interface.world_ptr.is_none() {
                return None;
            }
            let world = interface.world_ptr.unwrap();
            if let Some(bridge) = interface.components_bridge.get_mut(name) {
                return Some(bridge.on_read_data(unsafe { world.as_ref().unwrap() }, entity));
            }
        }
        None
    }

    pub(crate) fn write_component_bridge(name: &str, entity: Entity, value: JsValue) {
        if let Ok(mut interface) = INTERFACE.lock() {
            if interface.world_ptr.is_none() {
                return;
            }
            let world = interface.world_ptr.unwrap();
            if let Some(bridge) = interface.components_bridge.get_mut(name) {
                bridge.on_write_data(unsafe { world.as_mut().unwrap() }, entity, value);
            }
        }
    }

    pub(crate) fn build_state(name: &str) -> Option<JsValue> {
        if let Ok(interface) = INTERFACE.lock() {
            if let Some(factory) = interface.state_factory.get(name) {
                if let Ok(result) = factory.call0(&JsValue::UNDEFINED) {
                    return Some(result);
                }
            }
        }
        None
    }

    pub(crate) fn build_state_scripted(name: &str) -> Option<Box<dyn WebScriptStateScripted>> {
        if let Ok(mut interface) = INTERFACE.lock() {
            if let Some(factory) = interface.scriptable_state_factory.get_mut(name) {
                return Some(factory());
            }
        }
        None
    }

    pub(crate) fn create_entity(data: JsValue) -> EntityId {
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

    pub(crate) fn destroy_entity(id: EntityId) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.entities_to_destroy.insert(id);
        }
    }

    pub(crate) fn run_systems() {
        let meta = if let Ok(mut interface) = INTERFACE.lock() {
            std::mem::replace(&mut interface.systems_cache, None)
        } else {
            return;
        };
        if let Some(meta) = &meta {
            for (context, on_run) in meta {
                drop(on_run.call0(&context));
            }
        }
        if let Ok(mut interface) = INTERFACE.lock() {
            std::mem::replace(&mut interface.systems_cache, meta);
        }
    }

    pub(crate) fn maintain_entities(world: &mut World) {
        if let Ok(mut interface) = INTERFACE.lock() {
            let entities_to_destroy =
                std::mem::replace(&mut interface.entities_to_destroy, HashSet::new());
            for id in entities_to_destroy {
                if let Some(entity) = interface.entities_map.remove(&id) {
                    interface.entities_cache.retain(|e| *e != entity);
                    drop(world.delete_entity(entity));
                }
            }

            let entities_to_create = std::mem::replace(&mut interface.entities_to_create, vec![]);
            for (data, id) in entities_to_create {
                interface.build_entity(world, id, data);
            }
        }
    }

    fn build_entity(&mut self, world: &mut World, id: EntityId, data: JsValue) {
        let mut builder = world.create_entity();
        let mut components = HashMap::new();
        if !data.is_null() && !data.is_undefined() {
            if let Some(object) = Object::try_from(&data) {
                let keys = Object::keys(&object)
                    .iter()
                    .map(|key| key.dyn_ref::<JsString>().map(|key| String::from(key)))
                    .collect::<Vec<_>>();
                let values = Object::values(&object).iter().collect::<Vec<_>>();
                for (key, value) in keys.into_iter().zip(values.into_iter()) {
                    if let Some(key) = key {
                        if let Some(factory) = self.component_factory.get(&key) {
                            if let Ok(v) = factory.call0(&JsValue::UNDEFINED) {
                                if let Some(d) = Object::try_from(&v) {
                                    let v = if let Some(o) = Object::try_from(&value) {
                                        Object::assign(d, o).into()
                                    } else {
                                        v
                                    };
                                    components.insert(key, Some(v));
                                }
                            }
                        } else if let Some(scriptable) = self.scriptable_components.get(&key) {
                            if let Ok(v) = scriptable.to_js() {
                                if let Some(d) = Object::try_from(&v) {
                                    let v = if let Some(o) = Object::try_from(&value) {
                                        Object::assign(d, o).into()
                                    } else {
                                        v
                                    };
                                    components.insert(key, Some(v));
                                }
                            }
                        } else if let Some(bridge) = self.components_bridge.get_mut(&key) {
                            builder = bridge.on_add_to_entity(value, builder);
                            components.insert(key, None);
                        }
                    }
                }
            }
        }
        builder = builder.with(WebScriptComponent::new(id, components));
        let entity = builder.build();
        self.entities_map.insert(id, entity);
        self.entities_cache.push(entity);
    }
}
