use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct AnimatedBackground {
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
    #[serde(default)]
    pub phase: Vec2,
    #[serde(default)]
    pub speed: Vec2,
}

impl Prefab for AnimatedBackground {}
impl PrefabComponent for AnimatedBackground {}
