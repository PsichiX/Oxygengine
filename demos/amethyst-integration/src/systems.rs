#![allow(clippy::type_complexity)]

use crate::components::PlayerTag;
use amethyst::{
    core::Time,
    ecs::{Join, Read, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{debug_drawing::DebugLinesComponent, palette::rgb::Srgba},
};
use nav::prelude::*;
use winit::MouseButton;

const COMMAND_DELAY: f32 = 0.0;

pub struct NavDriverSystem;

impl<'s> System<'s> for NavDriverSystem {
    type SystemData = (
        Read<'s, Time>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, SimpleNavDriverTag>,
    );

    fn run(&mut self, (time, agents, drivers): Self::SystemData) {
        SimpleNavDriverSystem::run_impl(time.delta_seconds() as Scalar, (agents, drivers));
    }
}

pub struct RenderSystem;

impl<'s> System<'s> for RenderSystem {
    type SystemData = (
        ReadStorage<'s, NavAgent>,
        WriteStorage<'s, DebugLinesComponent>,
    );

    fn run(&mut self, (agents, mut debugs): Self::SystemData) {
        for (agent, debug) in (&agents, &mut debugs).join() {
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

#[derive(Default)]
pub struct CommandAgentsSystem {
    mouse_left_cooldown: f32,
}

impl<'s> System<'s> for CommandAgentsSystem {
    type SystemData = (
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, NavMeshesRes>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, PlayerTag>,
    );

    fn run(&mut self, (time, input, meshes, mut agents, players): Self::SystemData) {
        let delta_time = time.delta_seconds();
        self.mouse_left_cooldown = (self.mouse_left_cooldown - delta_time).max(0.0);
        for (agent, _) in (&mut agents, &players).join() {
            if input.mouse_button_is_down(MouseButton::Left) && self.mouse_left_cooldown <= 0.0 {
                if let Some((mut x, mut y)) = input.mouse_position() {
                    x = x.max(0.0).min(800.0);
                    y = 600.0 - y.max(0.0).min(600.0);
                    let mesh = meshes.meshes_iter().nth(0).unwrap().id();
                    self.mouse_left_cooldown = COMMAND_DELAY;
                    agent.set_destination(
                        NavAgentTarget::Point((x as Scalar, y as Scalar).into()),
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
                    agent.position.x = x as Scalar;
                    agent.position.y = y as Scalar;
                    agent.position.z = 0.0;
                }
            }
        }
    }
}
