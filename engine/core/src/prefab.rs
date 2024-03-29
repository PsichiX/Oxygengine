use crate::{
    app::{AppBuilder, AppLifeCycle},
    assets::{asset::AssetId, database::AssetsDatabase, protocols::prefab::PrefabAsset},
    ecs::{
        components::{Name, NonPersistent, NonPersistentPrefabProxy, Tag},
        hierarchy::{Parent, ParentPrefabProxy},
        life_cycle::EntityChanges,
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Universe,
    },
    state::StateToken,
};
use hecs::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

pub use serde_json::Value as PrefabValue;

type ComponentFactory = Box<
    dyn Fn(
            &mut EntityBuilder,
            &PrefabValue,
            &HashMap<String, Entity>,
            StateToken,
        ) -> Result<(), PrefabError>
        + Send
        + Sync,
>;

#[derive(Debug)]
pub enum PrefabError {
    CouldNotSerialize(String),
    CouldNotDeserialize(String),
    Custom(String),
}

pub trait Prefab: Serialize + DeserializeOwned + Sized {
    fn from_prefab(data: &PrefabValue) -> Result<Self, PrefabError> {
        match serde_json::from_value(data.clone()) {
            Ok(result) => {
                let mut result: Self = result;
                result.post_from_prefab();
                Ok(result)
            }
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn from_prefab_with_extras(
        data: &PrefabValue,
        _named_entities: &HashMap<String, Entity>,
        _state_token: StateToken,
    ) -> Result<Self, PrefabError> {
        Self::from_prefab(data)
    }

    fn to_prefab(&self) -> Result<PrefabValue, PrefabError> {
        match serde_json::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn from_prefab_str(data: &str) -> Result<Self, PrefabError> {
        match serde_json::from_str(data) {
            Ok(result) => {
                let mut result: Self = result;
                result.post_from_prefab();
                Ok(result)
            }
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn to_prefab_string(&self) -> Result<String, PrefabError> {
        match serde_json::to_string_pretty(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotSerialize(error.to_string())),
        }
    }

    fn post_from_prefab(&mut self) {}
}

pub trait PrefabProxy<P>: Component + Sized
where
    P: Prefab,
{
    fn from_proxy_with_extras(
        proxy: P,
        named_entities: &HashMap<String, Entity>,
        state_token: StateToken,
    ) -> Result<Self, PrefabError>;
}

impl Prefab for PrefabValue {}
pub trait PrefabComponent: Prefab + Component {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PrefabScene {
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub entities: Vec<PrefabSceneEntity>,
}

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

impl Prefab for PrefabScene {}
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
    component_factory: HashMap<String, ComponentFactory>,
    templates: HashMap<String, PrefabScene>,
}

impl PrefabManager {
    pub fn register_component_factory<T>(&mut self, name: &str)
    where
        T: PrefabComponent,
    {
        self.component_factory.insert(
            name.to_owned(),
            Box::new(|builder, prefab, named_entities, state_token| {
                builder.add(T::from_prefab_with_extras(
                    prefab,
                    named_entities,
                    state_token,
                )?);
                Ok(())
            }),
        );
    }

    pub fn register_component_factory_proxy<T, P>(&mut self, name: &str)
    where
        P: Prefab,
        T: PrefabProxy<P>,
    {
        self.component_factory.insert(
            name.to_owned(),
            Box::new(|builder, prefab, named_entities, state_token| {
                let p = P::from_prefab(prefab)?;
                builder.add(T::from_proxy_with_extras(p, named_entities, state_token)?);
                Ok(())
            }),
        );
    }

    pub fn unregister_component_factory(&mut self, name: &str) {
        self.component_factory.remove(name);
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

    pub fn instantiate(
        &mut self,
        name: &str,
        universe: &mut Universe,
    ) -> Result<Vec<Entity>, PrefabError> {
        let state_token = universe
            .expect_resource::<AppLifeCycle>()
            .current_state_token();
        let mut world = universe.world_mut();
        let mut changes = universe.expect_resource_mut::<EntityChanges>();
        self.instantiate_direct(name, &mut world, &mut changes, state_token)
    }

    pub fn instantiate_direct(
        &mut self,
        name: &str,
        world: &mut World,
        changes: &mut EntityChanges,
        state_token: StateToken,
    ) -> Result<Vec<Entity>, PrefabError> {
        Ok(self
            .build_template(name, world, changes, state_token, &Default::default())?
            .0)
    }

    pub fn load_scene_from_prefab(
        &mut self,
        prefab: &PrefabScene,
        universe: &mut Universe,
    ) -> Result<Vec<Entity>, PrefabError> {
        let state_token = universe
            .expect_resource::<AppLifeCycle>()
            .current_state_token();
        self.load_scene_from_prefab_direct(
            prefab,
            &mut universe.world_mut(),
            &mut universe.expect_resource_mut::<EntityChanges>(),
            state_token,
        )
    }

    pub fn load_scene_from_prefab_direct(
        &mut self,
        prefab: &PrefabScene,
        world: &mut World,
        changes: &mut EntityChanges,
        state_token: StateToken,
    ) -> Result<Vec<Entity>, PrefabError> {
        Ok(self
            .load_scene_from_prefab_inner(prefab, world, changes, state_token, &Default::default())?
            .0)
    }

    fn load_scene_from_prefab_inner(
        &mut self,
        prefab: &PrefabScene,
        world: &mut World,
        changes: &mut EntityChanges,
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
                        world,
                        changes,
                        state_token,
                        &named_entities,
                    )?;
                    if let Some(uid) = &data.uid {
                        named_entities.insert(uid.to_owned(), entity);
                    }
                    result_entities.push(entity);
                }
                PrefabSceneEntity::Template(name) => {
                    let (entities, uids) =
                        self.build_template(name, world, changes, state_token, &named_entities)?;
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
        world: &mut World,
        changes: &mut EntityChanges,
        state_token: StateToken,
        named_entities: &HashMap<String, Entity>,
    ) -> Result<Entity, PrefabError> {
        let mut entity_builder = EntityBuilder::new();
        for (key, component_meta) in components {
            if let Some(factory) = self.component_factory.get_mut(key) {
                factory(
                    &mut entity_builder,
                    component_meta,
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
        let entity = world.reserve_entity();
        let components = entity_builder.build();
        world.spawn_at(entity, components);
        changes.spawned.insert(entity);
        let components = changes.added_components.entry(entity).or_default();
        components.extend(entity_builder.component_types());
        Ok(entity)
    }

    fn build_template(
        &mut self,
        name: &str,
        world: &mut World,
        changes: &mut EntityChanges,
        state_token: StateToken,
        named_entities: &HashMap<String, Entity>,
    ) -> Result<(Vec<Entity>, HashMap<String, Entity>), PrefabError> {
        if let Some(prefab) = self.templates.get(name).cloned() {
            self.load_scene_from_prefab_inner(&prefab, world, changes, state_token, named_entities)
        } else {
            Err(PrefabError::Custom(format!(
                "There is no template registered: {}",
                name
            )))
        }
    }
}

#[derive(Default)]
pub struct PrefabSystemCache {
    templates_table: HashMap<AssetId, String>,
}
pub type PrefabSystemResources<'a> = (
    &'a AssetsDatabase,
    &'a mut PrefabManager,
    &'a mut PrefabSystemCache,
);

pub fn prefab_system(universe: &mut Universe) {
    let (assets, mut prefabs, mut cache) = universe.query_resources::<PrefabSystemResources>();

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
                cache.templates_table.insert(id, name.to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("prefab") {
        if let Some(name) = cache.templates_table.remove(id) {
            prefabs.unregister_scene_template(&name);
        }
    }
}

pub fn bundle_installer<PB, PMS>(
    builder: &mut AppBuilder<PB>,
    mut prefab_manager_setup: PMS,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
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
    builder.install_resource(PrefabSystemCache::default());
    builder.install_system::<PrefabSystemResources>("prefab", prefab_system, &[])?;
    Ok(())
}
