pub mod camera;
pub mod gizmo;
pub mod material_instance;
pub mod mesh_instance;
pub mod sprite_animation_instance;
pub mod text_instance;
pub mod tilemap_instance;
pub mod transform;
pub mod virtual_image_uniforms;
pub mod visibility;

use crate::mesh::BufferStorage;
use core::Ignite;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
