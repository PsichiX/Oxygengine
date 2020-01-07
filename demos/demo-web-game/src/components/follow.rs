use oxygengine::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum FollowMode {
    Instant,
    Delayed(f32),
}

#[derive(Debug, Copy, Clone)]
pub struct Follow(pub Option<Entity>, pub FollowMode);

impl Component for Follow {
    type Storage = VecStorage<Self>;
}
