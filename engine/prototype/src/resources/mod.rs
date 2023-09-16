pub mod audio_player;
pub mod camera;
pub mod renderables;
pub mod spatial_queries;

use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Transform2d {
    pub position: Vec2,
    pub rotation: Scalar,
    pub scale: Vec2,
}

impl Default for Transform2d {
    fn default() -> Self {
        Self {
            position: 0.0.into(),
            rotation: 0.0,
            scale: 1.0.into(),
        }
    }
}

impl Transform2d {
    pub fn position(mut self, value: impl Into<Vec2>) -> Self {
        self.position = value.into();
        self
    }

    pub fn rotation(mut self, value: Scalar) -> Self {
        self.rotation = value;
        self
    }

    pub fn scale(mut self, value: impl Into<Vec2>) -> Self {
        self.scale = value.into();
        self
    }
}

impl From<Transform2d> for HaTransform {
    fn from(other: Transform2d) -> Self {
        HaTransform::new(
            other.position.into(),
            Eulers::yaw(other.rotation),
            other.scale.with_z(1.0),
        )
    }
}
