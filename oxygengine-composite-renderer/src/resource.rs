use crate::{
    component::{UiImagePath, UiMargin},
    math::{Mat2d, Rect, Vec2},
};
use core::{
    ecs::{storage::UnprotectedStorage, world::Index, BitSet, DenseVecStorage, Entity, Join},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

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

#[derive(Debug, Default)]
pub struct CompositeUiInteractibles {
    /// {name: screen rect}
    pub(crate) bounding_boxes: HashMap<Cow<'static, str>, Rect>,
}

impl CompositeUiInteractibles {
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.bounding_boxes.keys().map(|key| key.as_ref())
    }

    pub fn find_rect(&self, name: &str) -> Option<Rect> {
        self.bounding_boxes.get(name).copied()
    }

    pub fn does_rect_contains_point(&self, name: &str, point: Vec2) -> bool {
        if let Some(rect) = self.bounding_boxes.get(name) {
            rect.contains_point(point)
        } else {
            false
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum UiValue {
    Value(Scalar),
    State(Cow<'static, str>),
    // (state name, source lower, source upper, target lower, target upper)
    MapState(Cow<'static, str>, Scalar, Scalar, Scalar, Scalar),
}

impl Default for UiValue {
    fn default() -> Self {
        Self::Value(0.0)
    }
}

impl From<Scalar> for UiValue {
    fn from(value: Scalar) -> Self {
        Self::Value(value)
    }
}

impl From<&str> for UiValue {
    fn from(value: &str) -> Self {
        Self::State(value.to_owned().into())
    }
}

impl From<(&str, Scalar, Scalar)> for UiValue {
    fn from(value: (&str, Scalar, Scalar)) -> Self {
        Self::MapState(value.0.to_owned().into(), 0.0, 1.0, value.1, value.2)
    }
}

impl From<(&str, Scalar, Scalar, Scalar, Scalar)> for UiValue {
    fn from(value: (&str, Scalar, Scalar, Scalar, Scalar)) -> Self {
        Self::MapState(
            value.0.to_owned().into(),
            value.1,
            value.2,
            value.3,
            value.4,
        )
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct UiValueVec2 {
    pub x: UiValue,
    pub y: UiValue,
}

impl UiValueVec2 {
    pub fn new(x: UiValue, y: UiValue) -> Self {
        Self { x, y }
    }
}

impl Default for UiValueVec2 {
    fn default() -> Self {
        Self::new(0.0.into(), 0.0.into())
    }
}

impl From<(UiValue, UiValue)> for UiValueVec2 {
    fn from((x, y): (UiValue, UiValue)) -> Self {
        Self::new(x, y)
    }
}

impl From<[UiValue; 2]> for UiValueVec2 {
    fn from([x, y]: [UiValue; 2]) -> Self {
        Self::new(x, y)
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum UiThemed {
    None,
    Image {
        #[serde(default)]
        source_rect: Rect,
        #[serde(default)]
        image_margin: UiMargin,
        #[serde(default)]
        image_path: UiImagePath,
        #[serde(default = "UiThemed::default_alpha")]
        alpha: UiValue,
    },
    Text {
        #[serde(default)]
        font_name: Cow<'static, str>,
        #[serde(default)]
        font_size: UiValue,
        #[serde(default = "UiThemed::default_alpha")]
        alpha: UiValue,
    },
}

impl Default for UiThemed {
    fn default() -> Self {
        Self::None
    }
}

impl UiThemed {
    fn default_alpha() -> UiValue {
        UiValue::Value(1.0)
    }
}

#[derive(Debug, Default)]
pub struct CompositeUiThemes {
    pub themes: HashMap<Cow<'static, str>, UiThemed>,
}
