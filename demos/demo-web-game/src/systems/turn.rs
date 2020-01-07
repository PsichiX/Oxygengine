#![allow(clippy::type_complexity)]

use crate::{
    components::follow::Follow,
    resources::{globals::Globals, turn::TurnManager},
};
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct TurnSystem;

impl<'s> System<'s> for TurnSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Write<'s, Globals>,
        Write<'s, TurnManager>,
        WriteStorage<'s, Follow>,
    );

    fn run(&mut self, (lifecycle, globals, mut turns, mut follows): Self::SystemData) {
        if !globals.phase.is_game() {
            return;
        }
        turns.process(lifecycle.delta_time_seconds());

        if let Some(camera) = globals.camera {
            if let Some(follow) = follows.get_mut(camera) {
                follow.0 = turns.selected();
            }
        }
    }
}
