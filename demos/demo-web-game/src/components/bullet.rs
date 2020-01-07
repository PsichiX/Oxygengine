use crate::components::player::PlayerType;
use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Bullet(pub PlayerType);

impl Component for Bullet {
    type Storage = VecStorage<Self>;
}
