use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Lifetime(pub f64);

impl Component for Lifetime {
    type Storage = VecStorage<Self>;
}
