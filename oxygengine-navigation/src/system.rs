use crate::{component::NavAgent, resource::NavMeshesRes};
use core::{
    app::AppLifeCycle,
    ecs::{Join, Read, ReadExpect, System, WriteStorage},
};

pub struct NavigationSystem;

impl<'s> System<'s> for NavigationSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        WriteStorage<'s, NavAgent>,
        Read<'s, NavMeshesRes>,
    );

    fn run(&mut self, (lifecycle, mut agents, meshes_res): Self::SystemData) {
        let delta_time = lifecycle.delta_time_seconds();
        for agent in (&mut agents).join() {
            agent.process(&meshes_res, delta_time);
        }
    }
}
