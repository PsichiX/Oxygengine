use crate::resources::*;
use oxygengine_core::prelude::*;

pub struct Camera {
    pub position: Vec2,
    pub rotation: Scalar,
    pub view_size: Scalar,
    pub(crate) viewport_size: Vec2,
    pub(crate) projection_matrix: Mat4,
    pub(crate) projection_matrix_inv: Mat4,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: 0.0,
            view_size: 1024.0,
            viewport_size: Default::default(),
            projection_matrix: Default::default(),
            projection_matrix_inv: Default::default(),
        }
    }
}

impl Camera {
    pub fn transform(&self) -> Transform2d {
        Transform2d {
            position: self.position,
            rotation: self.rotation,
            scale: 1.0.into(),
        }
    }

    pub fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    pub fn world_size(&self) -> Vec2 {
        self.projection_matrix_inv
            .mul_point(vec3(2.0, -2.0, 0.0))
            .into()
    }

    pub fn screen_to_camera_point(&self, mut point: Vec2) -> Vec2 {
        point.y = -point.y;
        self.projection_matrix_inv
            .mul_point(2.0 * point / self.viewport_size)
    }

    pub fn camera_to_screen_point(&self, point: Vec2) -> Vec2 {
        self.projection_matrix.mul_point(point) * self.viewport_size * vec2(0.5, -0.5)
    }
}
