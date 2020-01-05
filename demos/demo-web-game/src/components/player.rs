use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerType {
    North,
    South,
}

#[derive(Debug, Copy, Clone)]
pub struct Player(pub PlayerType);

impl Component for Player {
    type Storage = VecStorage<Self>;
}
