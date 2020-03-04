use oxygengine::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
/// (linear, angular)
pub struct Speed(pub Scalar, pub Scalar);

impl Component for Speed {
    type Storage = VecStorage<Self>;
}
