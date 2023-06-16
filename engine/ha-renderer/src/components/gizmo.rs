use crate::math::Rgba;
use core::prefab::{Prefab, PrefabComponent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct HaGizmo {
    #[serde(default)]
    pub visible: bool,
    #[serde(default = "HaGizmo::default_color")]
    pub color: Rgba,
}

impl HaGizmo {
    fn default_color() -> Rgba {
        Rgba::new(1.0, 0.0, 1.0, 1.0)
    }
}

impl Prefab for HaGizmo {}
impl PrefabComponent for HaGizmo {}
