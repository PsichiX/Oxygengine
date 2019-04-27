use oxygengine::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct FollowMouseTag;

impl Component for FollowMouseTag {
    type Storage = NullStorage<Self>;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct KeyboardMovementTag;

impl Component for KeyboardMovementTag {
    type Storage = NullStorage<Self>;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Speed(pub Scalar);

impl Component for Speed {
    type Storage = VecStorage<Self>;
}
