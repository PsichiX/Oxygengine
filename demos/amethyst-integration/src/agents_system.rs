use crate::components::PlayerTag;
use amethyst::{
    core::Time,
    ecs::{world::EntitiesRes, Join, Read, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{debug_drawing::DebugLinesComponent, palette::rgb::Srgba},
};
use nav::prelude::*;
use winit::MouseButton;

#[derive(Default)]
pub struct AgentsSystem {
    mouse_left_cooldown: f32,
}

impl<'s> System<'s> for AgentsSystem {
    type SystemData = (
        Read<'s, EntitiesRes>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
        WriteStorage<'s, NavAgent>,
        Read<'s, NavMeshesRes>,
        WriteStorage<'s, DebugLinesComponent>,
        ReadStorage<'s, PlayerTag>,
        Read<'s, NavMeshesRes>,
    );

    fn run(
        &mut self,
        (entities, time, input, mut agents, meshes_res, mut debugs, players, meshes): Self::SystemData,
    ) {
        let delta_time = time.delta_seconds();
        self.mouse_left_cooldown = (self.mouse_left_cooldown - delta_time).max(0.0);
        for (entity, agent, debug) in (&*entities, &mut agents, &mut debugs).join() {
            if players.get(entity).is_some() {
                if input.mouse_button_is_down(MouseButton::Left) && self.mouse_left_cooldown <= 0.0
                {
                    if let Some((mut x, mut y)) = input.mouse_position() {
                        x = x.max(0.0).min(800.0);
                        y = 600.0 - y.max(0.0).min(600.0);
                        println!("CLICK: {} x {}", x, y);
                        let mesh = meshes.meshes_iter().nth(0).unwrap().id();
                        self.mouse_left_cooldown = 0.1;
                        agent.set_destination(
                            (x as f64, y as f64).into(),
                            NavQuery::Accuracy,
                            NavPathMode::Accuracy,
                            mesh,
                        );
                    }
                } else if input.mouse_button_is_down(MouseButton::Right) {
                    agent.clear_path();
                } else if input.mouse_button_is_down(MouseButton::Middle) {
                    if let Some((mut x, mut y)) = input.mouse_position() {
                        x = x.max(0.0).min(800.0);
                        y = 600.0 - y.max(0.0).min(600.0);
                        println!("WARP: {} x {}", x, y);
                        agent.position.x = x as f64;
                        agent.position.y = y as f64;
                        agent.position.z = 0.0;
                    }
                }
            }

            agent.process(&meshes_res, delta_time as f64);

            debug.clear();
            if let Some(path) = agent.path() {
                for pair in path.windows(2) {
                    let f = pair[0];
                    let t = pair[1];
                    debug.add_line(
                        [f.x as f32, f.y as f32, f.z as f32].into(),
                        [t.x as f32, t.y as f32, t.z as f32].into(),
                        Srgba::new(0.0, 1.0, 0.0, 1.0),
                    );
                }
            }
            debug.add_circle_2d(
                [
                    agent.position.x as f32,
                    agent.position.y as f32,
                    agent.position.z as f32,
                ]
                .into(),
                20.0,
                6,
                Srgba::new(1.0, 0.0, 0.0, 1.0),
            );
        }
    }
}
