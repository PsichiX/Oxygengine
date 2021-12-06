use crate::ecs::Entity;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Default)]
pub struct EntityChanges {
    pub skip_clearing: bool,
    pub(crate) entities: HashSet<Entity>,
    pub(crate) spawned: HashSet<Entity>,
    pub(crate) despawned: HashSet<Entity>,
    pub(crate) added_components: HashMap<Entity, HashSet<TypeId>>,
    pub(crate) removed_components: HashMap<Entity, HashSet<TypeId>>,
}

impl EntityChanges {
    pub fn has_changed(&self) -> bool {
        !self.spawned.is_empty() || !self.despawned.is_empty()
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }

    pub fn spawned(&self) -> impl Iterator<Item = Entity> + '_ {
        self.spawned.iter().copied()
    }

    pub fn despawned(&self) -> impl Iterator<Item = Entity> + '_ {
        self.despawned.iter().copied()
    }

    pub fn added_components(&self, entity: Entity) -> Option<impl Iterator<Item = TypeId> + '_> {
        self.added_components
            .get(&entity)
            .map(|types| types.iter().copied())
    }

    pub fn has_added_component<T: 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.added_components
            .get(&entity)
            .map(|types| types.iter().any(|t| t == &type_id))
            .unwrap_or_default()
    }

    pub fn removed_components(&self, entity: Entity) -> Option<impl Iterator<Item = TypeId> + '_> {
        self.removed_components
            .get(&entity)
            .map(|types| types.iter().copied())
    }

    pub fn has_removed_component<T: 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.removed_components
            .get(&entity)
            .map(|types| types.iter().any(|t| t == &type_id))
            .unwrap_or_default()
    }

    pub(crate) fn clear(&mut self) {
        if self.skip_clearing {
            self.skip_clearing = false;
            return;
        }
        self.entities.clear();
        self.spawned.clear();
        self.despawned.clear();
        self.added_components.clear();
        self.removed_components.clear();
    }
}
