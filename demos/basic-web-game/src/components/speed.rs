use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

// component that tells the speed of entity.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Speed(pub Scalar);

impl Component for Speed {
    // not all entities has speed so we use `VecStorage`.
    type Storage = VecStorage<Self>;
}

impl Prefab for Speed {}
impl PrefabComponent for Speed {}
