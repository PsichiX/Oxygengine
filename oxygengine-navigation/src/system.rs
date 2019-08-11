use crate::{
    component::{NavAgent, NavAgentTarget, SimpleNavDriverTag},
    resource::{NavMesh, NavMeshesRes, NavVec3},
    Scalar,
};
use core::{
    app::AppLifeCycle,
    ecs::{Entities, Entity, Join, Read, ReadExpect, ReadStorage, System, WriteStorage},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct NavAgentMaintainSystem(HashMap<Entity, Vec<NavVec3>>);

impl NavAgentMaintainSystem {
    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
}

impl<'s> System<'s> for NavAgentMaintainSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, NavMeshesRes>,
        WriteStorage<'s, NavAgent>,
    );

    fn run(&mut self, (entities, meshes_res, mut agents): Self::SystemData) {
        self.0.clear();
        for (entity, agent) in (&entities, &agents).join() {
            if agent.dirty_path {
                if let Some(destination) = &agent.destination {
                    if let Some(mesh) = meshes_res.0.get(&destination.mesh) {
                        match destination.target {
                            NavAgentTarget::Point(point) => {
                                if let Some(path) = mesh.find_path(
                                    agent.position,
                                    point,
                                    destination.query,
                                    destination.mode,
                                ) {
                                    self.0.insert(entity, path);
                                }
                            }
                            NavAgentTarget::Entity(entity) => {
                                if let Some(other) = agents.get(entity) {
                                    if let Some(path) = mesh.find_path(
                                        agent.position,
                                        other.position,
                                        destination.query,
                                        destination.mode,
                                    ) {
                                        self.0.insert(entity, path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for (entity, path) in self.0.drain() {
            if let Some(agent) = agents.get_mut(entity) {
                agent.set_path(path);
            }
        }
    }
}

pub struct SimpleNavDriverSystem;

impl SimpleNavDriverSystem {
    pub fn run_impl<'s>(
        delta_time: Scalar,
        (mut agents, drivers): (
            WriteStorage<'s, NavAgent>,
            ReadStorage<'s, SimpleNavDriverTag>,
        ),
    ) {
        if delta_time <= 0.0 {
            return;
        }
        for (agent, _) in (&mut agents, &drivers).join() {
            if let Some(path) = agent.path() {
                if let Some((target, _)) = NavMesh::path_target_point(
                    path,
                    agent.position,
                    agent.speed.max(agent.min_target_distance.max(0.0)) * delta_time,
                ) {
                    let diff = target - agent.position;
                    let dir = diff.normalize();
                    agent.position = agent.position
                        + dir * (agent.speed.max(0.0) * delta_time).min(diff.magnitude());
                    agent.direction = diff.normalize();
                }
            }
        }
    }
}

impl<'s> System<'s> for SimpleNavDriverSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, SimpleNavDriverTag>,
    );

    fn run(&mut self, (lifecycle, agents, drivers): Self::SystemData) {
        let delta_time = lifecycle.delta_time_seconds();
        Self::run_impl(delta_time, (agents, drivers));
    }
}
