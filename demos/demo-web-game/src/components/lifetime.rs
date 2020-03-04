use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Lifetime(pub Scalar);

impl Component for Lifetime {
    type Storage = VecStorage<Self>;
}
