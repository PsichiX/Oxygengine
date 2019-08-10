use crate::{
    component::{NavAgent, NavAgentTarget, SimpleNavDriverTag},
    resource::{NavMesh, NavMeshesRes},
};
use core::{
    app::AppLifeCycle,
    ecs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage},
};

pub struct NavAgentMaintainSystem;

impl<'s> System<'s> for NavAgentMaintainSystem {
    type SystemData = (
        Read<'s, NavMeshesRes>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, NavAgent>,
    );

    fn run(&mut self, (meshes_res, mut agents, agents2): Self::SystemData) {
        for agent in (&mut agents).join() {
            if agent.dirty_path {
                if let Some(destination) = &agent.destination {
                    if let Some(mesh) = meshes_res.0.get(&destination.mesh) {
                        match destination.target {
                            NavAgentTarget::Point(point) => {
                                agent.path = mesh.find_path(
                                    agent.position,
                                    point,
                                    destination.query,
                                    destination.mode,
                                );
                                agent.dirty_path = false;
                            }
                            NavAgentTarget::Entity(entity) => {
                                if let Some(other) = agents2.get(entity) {
                                    agent.path = mesh.find_path(
                                        agent.position,
                                        other.position,
                                        destination.query,
                                        destination.mode,
                                    );
                                    agent.dirty_path = false;
                                }
                            }
                        }
                    } else {
                        agent.destination = None;
                        agent.dirty_path = false;
                    }
                } else {
                    agent.dirty_path = false;
                }
            }
        }
    }
}

pub struct SimpleNavDriverSystem;

impl<'s> System<'s> for SimpleNavDriverSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, SimpleNavDriverTag>,
    );

    fn run(&mut self, (lifecycle, mut agents, drivers): Self::SystemData) {
        let delta_time = lifecycle.delta_time_seconds();
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
