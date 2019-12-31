use oxygengine::prelude::*;

// component that tells the speed of entity.
#[derive(Debug, Default, Copy, Clone)]
pub struct Speed(pub f64);

impl Component for Speed {
    // not all entities has speed so we use `VecStorage`.
    type Storage = VecStorage<Self>;
}
