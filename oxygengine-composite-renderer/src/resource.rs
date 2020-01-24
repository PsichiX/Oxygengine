use crate::math::{Mat2d, Vec2};
use core::ecs::{storage::UnprotectedStorage, world::Index, BitSet, DenseVecStorage, Entity, Join};
use std::collections::HashMap;

#[derive(Default)]
pub struct CompositeTransformRes {
    mask: BitSet,
    inner: DenseVecStorage<Mat2d>,
    inner_inverse: DenseVecStorage<Mat2d>,
}

impl CompositeTransformRes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> CompositeTransformJoinable {
        CompositeTransformJoinable {
            mask: &self.mask,
            storage: &self.inner,
        }
    }

    pub fn write(&mut self) -> CompositeTransformJoinableMut {
        CompositeTransformJoinableMut {
            mask: &self.mask,
            storage: &mut self.inner,
        }
    }

    pub fn read_inverse(&self) -> CompositeTransformJoinable {
        CompositeTransformJoinable {
            mask: &self.mask,
            storage: &self.inner_inverse,
        }
    }

    pub fn write_inverse(&mut self) -> CompositeTransformJoinableMut {
        CompositeTransformJoinableMut {
            mask: &self.mask,
            storage: &mut self.inner_inverse,
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, matrix: Mat2d) {
        let id = entity.id();
        if self.mask.contains(id) {
            unsafe {
                *self.inner.get_mut(id) = matrix;
                *self.inner_inverse.get_mut(id) = (!matrix).unwrap_or_default();
            }
        } else {
            unsafe {
                self.inner.insert(id, matrix);
                self.inner_inverse.insert(id, (!matrix).unwrap_or_default());
            }
            self.mask.add(entity.id());
        }
    }

    pub(crate) fn clear(&mut self) {
        for id in &self.mask {
            unsafe {
                self.inner.remove(id);
                self.inner_inverse.remove(id);
            }
        }
        self.mask.clear();
    }
}

pub struct CompositeTransformJoinableMut<'a> {
    mask: &'a BitSet,
    storage: &'a mut DenseVecStorage<Mat2d>,
}

impl<'a> Join for CompositeTransformJoinableMut<'a> {
    type Mask = &'a BitSet;
    type Type = &'a mut Mat2d;
    type Value = &'a mut DenseVecStorage<Mat2d>;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (self.mask, self.storage)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        let value: *mut Self::Value = value as *mut Self::Value;
        (*value).get_mut(id)
    }
}

pub struct CompositeTransformJoinable<'a> {
    mask: &'a BitSet,
    storage: &'a DenseVecStorage<Mat2d>,
}

impl<'a> Join for CompositeTransformJoinable<'a> {
    type Mask = &'a BitSet;
    type Type = &'a Mat2d;
    type Value = &'a DenseVecStorage<Mat2d>;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (self.mask, self.storage)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        value.get(id)
    }
}

#[derive(Default)]
pub struct CompositeCameraCache {
    pub(crate) last_screen_size: Vec2,
    pub(crate) world_transforms: HashMap<Entity, Mat2d>,
    pub(crate) world_inverse_transforms: HashMap<Entity, Mat2d>,
}

impl CompositeCameraCache {
    pub fn last_screen_size(&self) -> Vec2 {
        self.last_screen_size
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
}
