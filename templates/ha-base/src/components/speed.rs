use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Speed(pub Scalar);

impl Prefab for Speed {}
impl PrefabComponent for Speed {}
