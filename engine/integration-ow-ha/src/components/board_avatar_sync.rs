use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct HaBoardAvatarSync {
    #[serde(default)]
    pub offset: Vec2,
    #[serde(default)]
    pub snap_to_pixel: bool,
}

impl Prefab for HaBoardAvatarSync {}

impl PrefabComponent for HaBoardAvatarSync {}
