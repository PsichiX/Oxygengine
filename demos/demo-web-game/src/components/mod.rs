use oxygengine::prelude::*;

pub mod speed;

// component that tags entity as moved with keyboard.
#[derive(Debug, Default, Copy, Clone)]
pub struct KeyboardMovementTag;

impl Component for KeyboardMovementTag {
    // tag components are empty so they use `NullStorage`.
    type Storage = NullStorage<Self>;
}
