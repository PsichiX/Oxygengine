use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

// component that tells the speed of entity.
#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Speed(pub Scalar);

impl Prefab for Speed {}
impl PrefabComponent for Speed {}
