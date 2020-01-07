use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Health(pub usize);

impl Component for Health {
    type Storage = VecStorage<Self>;
}
