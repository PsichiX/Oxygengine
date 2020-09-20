use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Triangles {
    pub image: String,
    pub size: Scalar,
}

impl Component for Triangles {
    type Storage = VecStorage<Self>;
}

impl Prefab for Triangles {}
impl PrefabComponent for Triangles {}
