use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Ammo(pub usize);

impl Component for Ammo {
    type Storage = VecStorage<Self>;
}
