#![allow(clippy::type_complexity)]

use crate::{components::follow::Follow, resources::{globals::Globals, turn::TurnManager}};
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct TurnSystem;

impl<'s> System<'s> for TurnSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Read<'s, InputController>,
        Write<'s, Globals>,
        Write<'s, TurnManager>,
        WriteStorage<'s, Follow>,
    );

    fn run(&mut self, (lifecycle, input, globals, mut turns, mut follows): Self::SystemData) {
        let next_player = input.trigger_or_default("next-player").is_pressed();
        if next_player {
            turns.select_next();
        }

        if let Some(camera) = globals.camera {
            if let Some(follow) = follows.get_mut(camera) {
                follow.0 = turns.selected();
            }
        }
    }
}
