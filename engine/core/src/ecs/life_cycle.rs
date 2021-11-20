use crate::ecs::{Entity, Universe, WorldRef};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct EntityChanges {
    entities: HashSet<Entity>,
    spawned: HashSet<Entity>,
    despawned: HashSet<Entity>,
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
}

pub type EntityLifeCycleSystemResources<'a> = (WorldRef, &'a mut EntityChanges);

pub fn entity_life_cycle_system(universe: &mut Universe) {
    let (world, mut changes) = universe.query_resources::<EntityLifeCycleSystemResources>();

    let old = std::mem::replace(
        &mut changes.entities,
        world.iter().map(|id| id.entity()).collect(),
    );
    changes.spawned = changes.entities.difference(&old).copied().collect();
    changes.despawned = old.difference(&changes.entities).copied().collect();
}
