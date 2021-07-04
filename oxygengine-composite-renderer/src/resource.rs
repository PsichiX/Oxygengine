use crate::math::{Mat2d, Rect, Vec2};
use core::ecs::Entity;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeTransformCache {
    matrix: HashMap<Entity, Mat2d>,
    matrix_inverse: HashMap<Entity, Mat2d>,
}

impl CompositeTransformCache {
    pub fn matrix(&self, entity: Entity) -> Option<Mat2d> {
        self.matrix.get(&entity).copied()
    }

    pub fn inverse_matrix(&self, entity: Entity) -> Option<Mat2d> {
        self.matrix_inverse.get(&entity).copied()
    }

    pub fn insert(&mut self, entity: Entity, matrix: Mat2d) {
        self.matrix.insert(entity, matrix);
        self.matrix_inverse
            .insert(entity, (!matrix).unwrap_or_default());
    }

    pub fn remove(&mut self, entity: Entity) {
        self.matrix.remove(&entity);
        self.matrix_inverse.remove(&entity);
    }

    pub fn clear(&mut self) {
        self.matrix.clear();
        self.matrix_inverse.clear();
    }
}

#[derive(Debug, Default)]
pub struct CompositeCameraCache {
    pub(crate) last_view_size: Vec2,
    pub(crate) world_transforms: HashMap<Entity, Mat2d>,
    pub(crate) world_inverse_transforms: HashMap<Entity, Mat2d>,
}

impl CompositeCameraCache {
    pub fn last_view_size(&self) -> Vec2 {
        self.last_view_size
    }

    pub fn screen_to_world_space(&self, entity: Entity, point: Vec2) -> Option<Vec2> {
        self.world_inverse_transforms
            .get(&entity)
            .map(|m| *m * point)
    }

    pub fn world_to_screen_space(&self, entity: Entity, point: Vec2) -> Option<Vec2> {
        self.world_transforms.get(&entity).map(|m| *m * point)
    }

    pub fn world_transform(&self, entity: Entity) -> Option<Mat2d> {
        self.world_transforms.get(&entity).cloned()
    }

    pub fn world_inverse_transform(&self, entity: Entity) -> Option<Mat2d> {
        self.world_inverse_transforms.get(&entity).cloned()
    }

    pub fn world_both_transforms(&self, entity: Entity) -> Option<(Mat2d, Mat2d)> {
        if let Some(t) = self.world_transforms.get(&entity) {
            if let Some(i) = self.world_inverse_transforms.get(&entity) {
                return Some((*t, *i));
            }
        }
        None
    }

    pub fn calculate_view_box(&self, entity: Entity) -> Option<Rect> {
        let m = self.world_inverse_transforms.get(&entity)?;
        let p1 = *m * Vec2::new(0.0, 0.0);
        let p2 = *m * Vec2::new(self.last_view_size.x, 0.0);
        let p3 = *m * self.last_view_size;
        let p4 = *m * Vec2::new(0.0, self.last_view_size.y);
        Rect::bounding(&[p1, p2, p3, p4])
    }

    pub fn calculate_world_size(&self, entity: Entity) -> Option<Vec2> {
        let m = self.world_inverse_transforms.get(&entity)?;
        let p1 = *m * Vec2::new(0.0, 0.0);
        let p2 = *m * Vec2::new(self.last_view_size.x, 0.0);
        let p3 = *m * Vec2::new(0.0, self.last_view_size.y);
        Some(Vec2::new((p2 - p1).magnitude(), (p3 - p1).magnitude()))
    }
}
