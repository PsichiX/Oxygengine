use crate::{
    app::{AppBuilder, AppLifeCycle},
    assets::{asset::AssetID, database::AssetsDatabase, protocols::prefab::PrefabAsset},
    hierarchy::{Name, NonPersistent, NonPersistentPrefabProxy, Parent, ParentPrefabProxy, Tag},
    state::StateToken,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use specs::{
    world::{Builder, EntitiesRes, LazyBuilder},
    Component, Entity, LazyUpdate, Read, ReadExpect, System, World, Write,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use serde_yaml::Value as PrefabValue;

type ComponentFactory = Box<
    dyn for<'a> FnMut(
            LazyBuilder<'a>,
            PrefabValue,
            &HashMap<String, Entity>,
            StateToken,
        ) -> Result<LazyBuilder<'a>, PrefabError>
        + Send,
>;

#[derive(Debug)]
pub enum PrefabError {
    CouldNotSerialize(String),
    CouldNotDeserialize(String),
    Custom(String),
}

pub trait Prefab: Serialize + DeserializeOwned + Sized {
    fn from_prefab(data: PrefabValue) -> Result<Self, PrefabError> {
        match serde_yaml::from_value(data) {
            Ok(result) => {
                let mut result: Self = result;
                result.post_from_prefab();
                Ok(result)
            }
            Err(error) => Err(PrefabError::CouldNotSerialize(error.to_string())),
        }
    }

    fn from_prefab_with_extras(
        data: PrefabValue,
        _named_entities: &HashMap<String, Entity>,
        _state_token: StateToken,
    ) -> Result<Self, PrefabError> {
        Self::from_prefab(data)
    }

    fn to_prefab(&self) -> Result<PrefabValue, PrefabError> {
        match serde_yaml::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn from_prefab_str(data: &str) -> Result<Self, PrefabError> {
        match serde_yaml::from_str(data) {
            Ok(result) => {
                let mut result: Self = result;
                result.post_from_prefab();
                Ok(result)
            }
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn to_prefab_string(&self) -> Result<String, PrefabError> {
        match serde_yaml::to_string(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotSerialize(error.to_string())),
        }
    }

    fn post_from_prefab(&mut self) {}
}

impl Prefab for PrefabValue {}

pub trait PrefabProxy<P>: Component
where
    P: Prefab,
{
    fn from_proxy_with_extras(
        proxy: P,
        named_entities: &HashMap<String, Entity>,
        state_token: StateToken,
    ) -> Result<Self, PrefabError>;
}

pub trait PrefabComponent: Prefab + Component + Send + Sync {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PrefabScene {
    #[serde(default)]
    pub autoload: bool,
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub entities: Vec<PrefabSceneEntity>,
}

impl Prefab for PrefabScene {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrefabSceneEntity {
    Data(PrefabSceneEntityData),
    Template(String),
}

impl Default for PrefabSceneEntity {
    fn default() -> Self {
        Self::Data(Default::default())
    }
}

impl Prefab for PrefabSceneEntity {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PrefabSceneEntityData {
    #[serde(default)]
    pub uid: Option<String>,
    #[serde(default)]
    pub components: HashMap<String, PrefabValue>,
}

impl Prefab for PrefabSceneEntityData {}

#[derive(Default)]
pub struct PrefabManager {
    component_factory: Arc<Mutex<HashMap<String, ComponentFactory>>>,
    templates: HashMap<String, PrefabScene>,
}

impl PrefabManager {
    pub fn register_component_factory<T>(&mut self, name: &str)
    where
        T: PrefabComponent,
    {
        if let Ok(mut component_factory) = self.component_factory.lock() {
            component_factory.insert(
                name.to_owned(),
                Box::new(move |builder, prefab, named_entities, state_token| {
                    let c =
                        T::from_prefab_with_extras(prefab.clone(), named_entities, state_token)?;
                    Ok(builder.with(c))
                }),
            );
        }
    }

    pub fn register_component_factory_proxy<T, P>(&mut self, name: &str)
    where
        P: Prefab,
        T: PrefabProxy<P> + Component + Send + Sync,
    {
        if let Ok(mut component_factory) = self.component_factory.lock() {
            component_factory.insert(
                name.to_owned(),
                Box::new(move |builder, prefab, named_entities, state_token| {
                    let p = P::from_prefab(prefab)?;
                    let c = T::from_proxy_with_extras(p, named_entities, state_token)?;
                    Ok(builder.with(c))
                }),
            );
        }
    }

    pub fn unregister_component_factory(&mut self, name: &str) {
        if let Ok(mut component_factory) = self.component_factory.lock() {
            component_factory.remove(name);
        }
    }

    pub fn register_scene_template(&mut self, prefab: PrefabScene) -> Result<(), PrefabError> {
        if let Some(name) = &prefab.template_name {
            if self.templates.contains_key(name) {
                Err(PrefabError::Custom(format!(
                    "There is already registered template: {}",
                    name
                )))
            } else {
                self.templates.insert(name.to_owned(), prefab);
                Ok(())
            }
        } else {
            Err(PrefabError::Custom(
                "Template prefabs must have set template name".to_owned(),
            ))
        }
    }

    pub fn unregister_scene_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    pub fn find_template(&self, name: &str) -> Option<&PrefabScene> {
        self.templates.get(name)
    }

    pub fn instantiate_world(
        &mut self,
        name: &str,
        world: &World,
    ) -> Result<Vec<Entity>, PrefabError> {
        let entities = world.read_resource::<EntitiesRes>();
        let lazy_update = world.read_resource::<LazyUpdate>();
        let state_token = world.read_resource::<AppLifeCycle>().current_state_token();
        Ok(self
            .build_template(
                name,
                &entities,
                &lazy_update,
                state_token,
                &Default::default(),
            )?
            .0)
    }

    pub fn instantiate_direct(
        &mut self,
        name: &str,
        entities: &EntitiesRes,
        lazy_update: &LazyUpdate,
        state_token: StateToken,
    ) -> Result<Vec<Entity>, PrefabError> {
        Ok(self
            .build_template(
                name,
                entities,
                lazy_update,
                state_token,
                &Default::default(),
            )?
            .0)
    }

    pub fn instantiate_system_data<'s>(
        &mut self,
        name: &str,
        (entities, lazy_update, lifecycle): &(
            Read<'s, EntitiesRes>,
            Read<'s, LazyUpdate>,
            ReadExpect<'s, AppLifeCycle>,
        ),
    ) -> Result<Vec<Entity>, PrefabError> {
        self.instantiate_direct(
            name,
            &entities,
            &lazy_update,
            lifecycle.current_state_token(),
        )
    }

    pub fn load_scene_from_prefab_world(
        &mut self,
        prefab: &PrefabScene,
        world: &World,
    ) -> Result<Vec<Entity>, PrefabError> {
        let entities = world.read_resource::<EntitiesRes>();
        let lazy_update = world.read_resource::<LazyUpdate>();
        let state_token = world.read_resource::<AppLifeCycle>().current_state_token();
        self.load_scene_from_prefab_direct(prefab, &entities, &lazy_update, state_token)
    }

    pub fn load_scene_from_prefab_direct(
        &mut self,
        prefab: &PrefabScene,
        entities: &EntitiesRes,
        lazy_update: &LazyUpdate,
        state_token: StateToken,
    ) -> Result<Vec<Entity>, PrefabError> {
        Ok(self
            .load_scene_from_prefab_inner(
                prefab,
                entities,
                lazy_update,
                state_token,
                &Default::default(),
            )?
            .0)
    }

    pub fn load_scene_from_prefab_system_data<'s>(
        &mut self,
        prefab: &PrefabScene,
        (entities, lazy_update, lifecycle): &(
            Read<'s, EntitiesRes>,
            Read<'s, LazyUpdate>,
            ReadExpect<'s, AppLifeCycle>,
        ),
    ) -> Result<Vec<Entity>, PrefabError> {
        self.load_scene_from_prefab_direct(
            prefab,
            &entities,
            &lazy_update,
            lifecycle.current_state_token(),
        )
    }

    fn load_scene_from_prefab_inner(
        &mut self,
        prefab: &PrefabScene,
        entities: &EntitiesRes,
        lazy_update: &LazyUpdate,
        state_token: StateToken,
        named_entities: &HashMap<String, Entity>,
    ) -> Result<(Vec<Entity>, HashMap<String, Entity>), PrefabError> {
        let mut named_entities = named_entities.clone();
        let mut result_entities = vec![];
        for entity_meta in &prefab.entities {
            match entity_meta {
                PrefabSceneEntity::Data(data) => {
                    let entity = self.build_entity(
                        &data.components,
                        entities,
                        lazy_update,
                        state_token,
                        &named_entities,
                    )?;
                    if let Some(uid) = &data.uid {
                        named_entities.insert(uid.to_owned(), entity);
                    }
                    result_entities.push(entity);
                }
                PrefabSceneEntity::Template(name) => {
                    let (entities, uids) = self.build_template(
                        name,
                        entities,
                        lazy_update,
                        state_token,
                        &named_entities,
                    )?;
                    for (uid, entity) in uids {
                        named_entities.insert(uid.to_owned(), entity);
                    }
                    result_entities.extend(entities);
                }
            }
        }
        Ok((result_entities, named_entities))
    }

    fn build_entity(
        &mut self,
        components: &HashMap<String, PrefabValue>,
        entities: &EntitiesRes,
        lazy_update: &LazyUpdate,
        state_token: StateToken,
        named_entities: &HashMap<String, Entity>,
    ) -> Result<Entity, PrefabError> {
        if let Ok(mut component_factory) = self.component_factory.lock() {
            let mut entity_builder = lazy_update.create_entity(entities);
            for (key, component_meta) in components {
                if let Some(factory) = component_factory.get_mut(key) {
                    entity_builder = (factory)(
                        entity_builder,
                        component_meta.clone(),
                        named_entities,
                        state_token,
                    )?;
                } else {
                    return Err(PrefabError::CouldNotDeserialize(format!(
                        "Could not find component factory: {}",
                        key
                    )));
                }
            }
            Ok(entity_builder.build())
        } else {
            Err(PrefabError::Custom(
                "Could not acquire lock on component factory".to_owned(),
            ))
        }
    }

    fn build_template(
        &mut self,
        name: &str,
        entities: &EntitiesRes,
        lazy_update: &LazyUpdate,
        state_token: StateToken,
        named_entities: &HashMap<String, Entity>,
    ) -> Result<(Vec<Entity>, HashMap<String, Entity>), PrefabError> {
        if let Some(prefab) = self.templates.get(name).cloned() {
            self.load_scene_from_prefab_inner(
                &prefab,
                entities,
                lazy_update,
                state_token,
                named_entities,
            )
        } else {
            Err(PrefabError::Custom(format!(
                "There is no template registered: {}",
                name
            )))
        }
    }
}

#[derive(Default)]
pub struct PrefabSystem {
    templates_table: HashMap<AssetID, String>,
}

impl<'s> System<'s> for PrefabSystem {
    type SystemData = (
        Read<'s, EntitiesRes>,
        Read<'s, LazyUpdate>,
        ReadExpect<'s, AppLifeCycle>,
        ReadExpect<'s, AssetsDatabase>,
        Write<'s, PrefabManager>,
    );

    fn run(&mut self, (entities, lazy_update, lifecycle, assets, mut prefabs): Self::SystemData) {
        for id in assets.lately_loaded_protocol("prefab") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded prefab asset");
            let asset = asset
                .get::<PrefabAsset>()
                .expect("trying to use non-prefab asset");
            if let Some(name) = &asset.get().template_name {
                if prefabs.register_scene_template(asset.get().clone()).is_ok() {
                    self.templates_table.insert(id, name.to_owned());
                }
            }
            if asset.get().autoload {
                drop(prefabs.load_scene_from_prefab_direct(
                    asset.get(),
                    &entities,
                    &lazy_update,
                    lifecycle.current_state_token(),
                ));
            }
        }
        for id in assets.lately_unloaded_protocol("prefab") {
            if let Some(name) = self.templates_table.remove(id) {
                prefabs.unregister_scene_template(&name);
            }
        }
    }
}

pub fn bundle_installer<'a, 'b, PMS>(
    builder: &mut AppBuilder<'a, 'b>,
    mut prefab_manager_setup: PMS,
) where
    PMS: FnMut(&mut PrefabManager),
{
    let mut manager = PrefabManager::default();
    manager.register_component_factory_proxy::<Parent, ParentPrefabProxy>("Parent");
    manager.register_component_factory::<Tag>("Tag");
    manager.register_component_factory::<Name>("Name");
    manager.register_component_factory_proxy::<NonPersistent, NonPersistentPrefabProxy>(
        "NonPersistent",
    );
    prefab_manager_setup(&mut manager);
    builder.install_resource(manager);
    builder.install_system(PrefabSystem::default(), "prefab", &[]);
}
