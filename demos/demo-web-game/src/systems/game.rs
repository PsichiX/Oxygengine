#![allow(clippy::type_complexity)]

use crate::{
    components::player::Player,
    resources::{
        globals::{GamePhase, Globals},
        turn::TurnManager,
    },
};
use oxygengine::prelude::*;

pub struct GameSystem;

impl<'s> System<'s> for GameSystem {
    type SystemData = (
        Write<'s, Globals>,
        Write<'s, TurnManager>,
        Read<'s, InputController>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (mut globals, mut turns, input, players): Self::SystemData) {
        match globals.phase {
            GamePhase::Start => {
                if input.trigger_or_default("fire").is_pressed() {
                    globals.phase = GamePhase::Game;
                    turns.select_nth(0);
                }
            }
            GamePhase::Game => {
                let players = players.join().map(|p| p.0).collect::<Vec<_>>();
                match players.len() {
                    0 => {
                        globals.phase = GamePhase::End(None);
                    }
                    1 => {
                        globals.phase = GamePhase::End(Some(players[0]));
                    }
                    _ => {}
                }
            }
            GamePhase::End(_) => {
                if input.trigger_or_default("fire").is_pressed() {
                    globals.phase = GamePhase::Restart;
                }
            }
            _ => {}
        }
    }
}
