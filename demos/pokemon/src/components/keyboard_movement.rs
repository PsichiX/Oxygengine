use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::East
    }
}

// component that tags entity as moved with keyboard.
#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct KeyboardMovement {
    pub direction: Direction,
    pub is_moving: bool,
}

impl Component for KeyboardMovement {
    type Storage = VecStorage<Self>;
}

impl Prefab for KeyboardMovement {}
impl PrefabComponent for KeyboardMovement {}
