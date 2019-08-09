use amethyst::ecs::{Component, NullStorage};

#[derive(Debug, Default, Copy, Clone)]
pub struct PlayerTag;

impl Component for PlayerTag {
    type Storage = NullStorage<Self>;
}
