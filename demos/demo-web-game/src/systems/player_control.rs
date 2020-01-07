#![allow(clippy::type_complexity)]

use crate::{
    components::{ammo::Ammo, player::Player, speed::Speed},
    resources::{
        globals::Globals,
        spawner::{Spawn, Spawner},
        turn::TurnManager,
    },
};
use oxygengine::prelude::*;

const BULLET_VELOCITY: f64 = 200.0;
const BULLET_OFFSET: f64 = 64.0;

pub struct PlayerControlSystem;

impl<'s> System<'s> for PlayerControlSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Globals>,
        Read<'s, InputController>,
        Write<'s, Physics2dWorld>,
        ReadExpect<'s, AppLifeCycle>,
        Read<'s, TurnManager>,
        Write<'s, Spawner>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Speed>,
        WriteStorage<'s, Ammo>,
        ReadStorage<'s, RigidBody2d>,
    );

    fn run(
        &mut self,
        (
            entities,
            globals,
            input,
            mut world,
            lifecycle,
            turns,
            mut spawner,
            players,
            speed,
            mut ammo,
            bodies,
        ): Self::SystemData,
    ) {
        if !globals.phase.is_game() {
            return;
        }

        let dt = lifecycle.delta_time_seconds();
        let hor =
            -input.axis_or_default("move-left") as f64 + input.axis_or_default("move-right") as f64;
        let ver =
            -input.axis_or_default("move-up") as f64 + input.axis_or_default("move-down") as f64;

        if let Some(active) = turns.selected_playing() {
            for (entity, player, speed, body) in (&entities, &players, &speed, &bodies).join() {
                if entity == active {
                    if let Some(handle) = body.handle() {
                        if let Some(body) = world.body_mut(handle) {
                            body.apply_local_force(
                                0,
                                &Force::from_slice(&[0.0, speed.0 * -ver * dt, hor * speed.1]),
                                ForceType::VelocityChange,
                                true,
                            );

                            if let Some(ammo) = ammo.get_mut(entity) {
                                if input.trigger_or_default("fire").is_pressed() && ammo.0 > 0 {
                                    ammo.0 -= 1;
                                    let isometry = body.position();
                                    let p = isometry.translation.vector;
                                    let r = isometry.rotation.angle();
                                    let o =
                                        isometry.transform_vector(&(Vector::y() * BULLET_OFFSET));
                                    let v =
                                        isometry.transform_vector(&(Vector::y() * BULLET_VELOCITY));
                                    spawner.spawn(Spawn::Bullet(
                                        p + o,
                                        r,
                                        player.0,
                                        Velocity::linear(v.x, v.y),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
