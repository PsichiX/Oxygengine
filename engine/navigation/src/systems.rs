use crate::{
    components::{NavAgent, NavAgentTarget, SimpleNavDriverTag},
    resources::{nav_meshes::NavMeshes, NavMesh},
};
use core::{
    app::AppLifeCycle,
    ecs::{Comp, Universe, WorldRef},
};

pub type NavAgentMaintainSystemResources<'a> = (WorldRef, &'a NavMeshes, Comp<&'a mut NavAgent>);

pub fn nav_agent_maintain_system(universe: &mut Universe) {
    let (world, meshes, ..) = universe.query_resources::<NavAgentMaintainSystemResources>();

    for (entity, agent) in world.query::<&mut NavAgent>().iter() {
        if agent.dirty_path {
            if let Some(destination) = &agent.destination {
                if let Some(mesh) = meshes.0.get(&destination.mesh) {
                    match destination.target {
                        NavAgentTarget::Point(point) => {
                            if let Some(path) = mesh.find_path(
                                agent.position,
                                point,
                                destination.query,
                                destination.mode,
                            ) {
                                agent.set_path(path);
                            }
                        }
                        NavAgentTarget::Entity(other) => {
                            if entity != other {
                                if let Ok(other) = unsafe { world.get_unchecked::<NavAgent>(other) }
                                {
                                    if let Some(path) = mesh.find_path(
                                        agent.position,
                                        other.position,
                                        destination.query,
                                        destination.mode,
                                    ) {
                                        agent.set_path(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub type SimpleNavDriverSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    Comp<&'a mut NavAgent>,
    Comp<&'a SimpleNavDriverTag>,
);

pub fn simple_nav_driver_system(universe: &mut Universe) {
    let (world, lifecycle, ..) = universe.query_resources::<SimpleNavDriverSystemResources>();

    let delta_time = lifecycle.delta_time_seconds();
    if delta_time <= 0.0 {
        return;
    }
    for (_, agent) in world
        .query::<&mut NavAgent>()
        .with::<SimpleNavDriverTag>()
        .iter()
    {
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
