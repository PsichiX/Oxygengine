use crate::components::player::PlayerType;
use oxygengine::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Spawn {
    /// (position, rotation, owner type, velocity)
    Bullet(Vector<Scalar>, Scalar, PlayerType, Velocity<Scalar>),
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DespawnEffect {
    None,
    Explode,
    ExplodeSmoke,
}

#[derive(Debug, Default)]
pub struct Spawner {
    spawns: Vec<Spawn>,
    despawns: HashMap<Entity, DespawnEffect>,
}

impl Spawner {
    pub fn spawn(&mut self, spawn: Spawn) {
        self.spawns.push(spawn);
    }

    pub fn despawn(&mut self, entity: Entity, effect: DespawnEffect) {
        self.despawns.insert(entity, effect);
    }

    pub fn take_spawns(&mut self) -> Vec<Spawn> {
        std::mem::replace(&mut self.spawns, vec![])
    }

    pub fn take_despawns(&mut self) -> Vec<(Entity, DespawnEffect)> {
        self.despawns.drain().collect::<Vec<_>>()
    }
}
