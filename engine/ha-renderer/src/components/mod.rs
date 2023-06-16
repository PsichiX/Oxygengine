pub mod camera;
pub mod gizmo;
pub mod immediate_batch;
pub mod material_instance;
pub mod mesh_instance;
pub mod postprocess;
pub mod rig_animation_instance;
pub mod rig_instance;
pub mod sprite_animation_instance;
pub mod text_instance;
pub mod tilemap_instance;
pub mod transform;
pub mod virtual_image_uniforms;
pub mod visibility;
pub mod volume;
pub mod volume_overlap;
pub mod volume_visibility;

use crate::mesh::BufferStorage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaChangeFrequency {
    Low,
    High,
    Stream,
}

impl Default for HaChangeFrequency {
    fn default() -> Self {
        Self::Low
    }
}

impl From<HaChangeFrequency> for BufferStorage {
    fn from(frequency: HaChangeFrequency) -> Self {
        match frequency {
            HaChangeFrequency::Low => BufferStorage::Static,
            HaChangeFrequency::High => BufferStorage::Dynamic,
            HaChangeFrequency::Stream => BufferStorage::Stream,
        }
    }
}
