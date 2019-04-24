use crate::math::Scalar;
use core::ecs::{storage::UnprotectedStorage, world::Index, BitSet, DenseVecStorage, Entity, Join};

#[derive(Default)]
pub struct CompositeTransformRes {
    mask: BitSet,
    inner: DenseVecStorage<[Scalar; 6]>,
}

impl CompositeTransformRes {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add(&mut self, entity: Entity, matrix: [Scalar; 6]) {
        let id = entity.id();
        if self.mask.contains(id) {
            unsafe {
                *self.inner.get_mut(id) = matrix;
            }
        } else {
            unsafe {
                self.inner.insert(id, matrix);
            }
            self.mask.add(entity.id());
        }
    }

    pub(crate) fn clear(&mut self) {
        for id in &self.mask {
            unsafe {
                self.inner.remove(id);
            }
        }
        self.mask.clear();
    }
}

impl<'a> Join for &'a mut CompositeTransformRes {
    type Mask = &'a BitSet;
    type Type = &'a mut [Scalar; 6];
    type Value = &'a mut DenseVecStorage<[Scalar; 6]>;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (&self.mask, &mut self.inner)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        let value: *mut Self::Value = value as *mut Self::Value;
        (*value).get_mut(id)
    }
}

impl<'a> Join for &'a CompositeTransformRes {
    type Mask = &'a BitSet;
    type Type = &'a [Scalar; 6];
    type Value = &'a DenseVecStorage<[Scalar; 6]>;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (&self.mask, &self.inner)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        value.get(id)
    }
}
