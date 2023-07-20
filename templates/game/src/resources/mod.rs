pub mod effects;
pub mod game_state_info;

use oxygengine::prelude::*;

#[derive(Debug)]
pub enum GlobalEvent {
    LevelUp(Entity, usize),
}
