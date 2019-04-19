use crate::{
    composite_renderer::Renderable,
    math::{Scalar, Vec2},
};
use core::ecs::{Component, VecStorage};

#[derive(Debug, Clone)]
pub struct CompositeRenderable(pub Renderable<'static>);

impl Component for CompositeRenderable {
    type Storage = VecStorage<Self>;
}

impl From<Renderable<'static>> for CompositeRenderable {
    fn from(value: Renderable<'static>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct CompositeRenderableStroke(pub Scalar);

impl Component for CompositeRenderableStroke {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeTransform {
    pub translation: Vec2,
    pub rotation: Scalar,
    pub scale: Vec2,
}

impl Default for CompositeTransform {
    fn default() -> Self {
        Self {
            translation: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
        }
    }
}

impl CompositeTransform {
    pub fn new(translation: Vec2, rotation: Scalar, scale: Vec2) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn translation(v: Vec2) -> Self {
        Self::default().with_translation(v)
    }

    pub fn rotation(v: Scalar) -> Self {
        Self::default().with_rotation(v)
    }

    pub fn scale(v: Vec2) -> Self {
        Self::default().with_scale(v)
    }

    pub fn with_translation(mut self, v: Vec2) -> Self {
        self.translation = v;
        self
    }

    pub fn with_rotation(mut self, v: Scalar) -> Self {
        self.rotation = v;
        self
    }

    pub fn with_scale(mut self, v: Vec2) -> Self {
        self.scale = v;
        self
    }
}

impl Component for CompositeTransform {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct CompositeRenderDepth(pub Scalar);

impl Component for CompositeRenderDepth {
    type Storage = VecStorage<Self>;
}
