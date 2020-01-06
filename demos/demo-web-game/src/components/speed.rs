use oxygengine::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
/// (linear, angular)
pub struct Speed(pub f64, pub f64);

impl Component for Speed {
    type Storage = VecStorage<Self>;
}
