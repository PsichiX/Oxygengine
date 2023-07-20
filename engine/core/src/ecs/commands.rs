use crate::{
    app::AppLifeCycle,
    ecs::{life_cycle::EntityChanges, Universe},
    prefab::PrefabManager,
};
pub use hecs::*;
use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

pub trait UniverseCommand: Send + Sync {
    fn run(&mut self, universe: &mut Universe);
}

struct ClosureUniverseCommand(Box<dyn FnMut(&mut Universe) + Send + Sync>);

impl UniverseCommand for ClosureUniverseCommand {
    fn run(&mut self, universe: &mut Universe) {
        (self.0)(universe);
    }
}

pub struct SpawnEntity {
    pub entity_builder: EntityBuilder,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn FnMut(&mut Universe, Entity) + Send + Sync>>,
}

impl SpawnEntity {
    pub fn new(entity_builder: EntityBuilder) -> Self {
        Self {
            entity_builder,
            on_complete: None,
        }
    }

    pub fn from_bundle<B>(bundle: B) -> Self
    where
        B: DynamicBundle,
    {
        let mut entity_builder = EntityBuilder::new();
        entity_builder.add_bundle(bundle);
        Self::new(entity_builder)
    }

    pub fn on_complete<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Universe, Entity) + Send + Sync + 'static,
    {
        self.on_complete = Some(Box::new(f));
        self
    }

    pub fn execute(&mut self, universe: &mut Universe) -> Entity {
        let entity = universe.world_mut().spawn(self.entity_builder.build());
        let mut changes = universe.expect_resource_mut::<EntityChanges>();
        changes.spawned.insert(entity);
        let components = changes.added_components.entry(entity).or_default();
        components.extend(self.entity_builder.component_types());
        if let Some(mut on_complete) = self.on_complete.take() {
            (on_complete)(universe, entity);
        }
        entity
    }
}

impl UniverseCommand for SpawnEntity {
    fn run(&mut self, universe: &mut Universe) {
        self.execute(universe);
    }
}

pub struct DespawnEntity(pub Entity);

impl UniverseCommand for DespawnEntity {
    fn run(&mut self, universe: &mut Universe) {
        if universe.world_mut().despawn(self.0).is_ok() {
            universe
                .expect_resource_mut::<EntityChanges>()
                .despawned
                .insert(self.0);
        }
    }
}

pub struct EntityAddComponent<C>
where
    C: Component + Send + Sync,
{
    pub entity: Entity,
    component: Option<C>,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn FnMut(&mut Universe, Entity) + Send + Sync>>,
}

impl<C> EntityAddComponent<C>
where
    C: Component + Send + Sync,
{
    pub fn new(entity: Entity, component: C) -> Self {
        Self {
            entity,
            component: Some(component),
            on_complete: None,
        }
    }

    pub fn on_complete<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Universe, Entity) + Send + Sync + 'static,
    {
        self.on_complete = Some(Box::new(f));
        self
    }
}

impl<C> UniverseCommand for EntityAddComponent<C>
where
    C: Component + Send + Sync + 'static,
{
    fn run(&mut self, universe: &mut Universe) {
        if let Some(component) = self.component.take() {
            if universe
                .world_mut()
                .insert_one(self.entity, component)
                .is_ok()
            {
                let mut changes = universe.expect_resource_mut::<EntityChanges>();
                let components = changes.added_components.entry(self.entity).or_default();
                components.insert(TypeId::of::<C>());
                if let Some(mut on_complete) = self.on_complete.take() {
                    (on_complete)(universe, self.entity);
                }
            }
        }
    }
}

pub struct EntityRemoveComponent<C>
where
    C: Component,
{
    pub entity: Entity,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn FnMut(&mut Universe, Entity) + Send + Sync>>,
    _phantom: PhantomData<fn() -> C>,
}

impl<C> EntityRemoveComponent<C>
where
    C: Component,
{
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            on_complete: None,
            _phantom: Default::default(),
        }
    }

    pub fn on_complete<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Universe, Entity) + Send + Sync + 'static,
    {
        self.on_complete = Some(Box::new(f));
        self
    }
}

impl<C> UniverseCommand for EntityRemoveComponent<C>
where
    C: Component,
{
    fn run(&mut self, universe: &mut Universe) {
        if universe.world_mut().remove_one::<C>(self.entity).is_ok() {
            let mut changes = universe.expect_resource_mut::<EntityChanges>();
            let components = changes.removed_components.entry(self.entity).or_default();
            components.insert(TypeId::of::<C>());
            if let Some(mut on_complete) = self.on_complete.take() {
                (on_complete)(universe, self.entity);
            }
        }
    }
}

/// (template name, add/override components)
pub struct InstantiatePrefab {
    pub name: String,
    pub components: HashMap<usize, EntityBuilder>,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn FnMut(&mut Universe, &[Entity]) + Send + Sync>>,
}

impl InstantiatePrefab {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            components: Default::default(),
            on_complete: None,
        }
    }

    pub fn components(mut self, index: usize, entity_builder: EntityBuilder) -> Self {
        self.components.insert(index, entity_builder);
        self
    }

    pub fn components_from_bundle<B>(mut self, index: usize, bundle: B) -> Self
    where
        B: DynamicBundle,
    {
        let mut entity_builder = EntityBuilder::new();
        entity_builder.add_bundle(bundle);
        self.components.insert(index, entity_builder);
        self
    }

    pub fn on_complete<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Universe, &[Entity]) + Send + Sync + 'static,
    {
        self.on_complete = Some(Box::new(f));
        self
    }

    pub fn execute(&mut self, universe: &mut Universe) -> Option<Vec<Entity>> {
        if let Some(mut prefabs) = universe.resource_mut::<PrefabManager>() {
            let mut world = universe.world_mut();
            let mut changes = universe.expect_resource_mut::<EntityChanges>();
            let state_token = universe
                .expect_resource::<AppLifeCycle>()
                .current_state_token();
            let entities = if let Ok(entities) =
                prefabs.instantiate_direct(&self.name, &mut world, &mut changes, state_token)
            {
                for (index, entity_builder) in &mut self.components {
                    if let Some(entity) = entities.get(*index) {
                        let _ = world.insert(*entity, entity_builder.build());
                    }
                }
                entities
            } else {
                return None;
            };
            drop(world);
            if let Some(mut on_complete) = self.on_complete.take() {
                (on_complete)(universe, &entities);
            }
            Some(entities)
        } else {
            None
        }
    }
}

impl UniverseCommand for InstantiatePrefab {
    fn run(&mut self, universe: &mut Universe) {
        self.execute(universe);
    }
}

pub struct UniverseCommands {
    queue: VecDeque<Box<dyn UniverseCommand>>,
    resize: usize,
}

impl Default for UniverseCommands {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            resize: 1024,
        }
    }
}

impl UniverseCommands {
    pub fn schedule<T>(&mut self, command: T)
    where
        T: UniverseCommand + 'static,
    {
        self.schedule_raw(Box::new(command));
    }

    pub fn schedule_raw(&mut self, command: Box<dyn UniverseCommand>) {
        if self.resize > 0 && self.queue.len() + 1 >= self.queue.capacity() {
            self.queue.reserve(self.resize);
        }
        self.queue.push_back(command);
    }

    pub fn schedule_fn<F>(&mut self, f: F)
    where
        F: FnMut(&mut Universe) + Send + Sync + 'static,
    {
        self.schedule(ClosureUniverseCommand(Box::new(f)));
    }

    pub fn execute(&mut self, universe: &mut Universe) {
        while let Some(mut command) = self.queue.pop_front() {
            command.run(universe);
        }
    }
}
