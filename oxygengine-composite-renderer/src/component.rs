use crate::{
    composite_renderer::{Effect, Renderable},
    math::{Mat2d, Scalar, Vec2},
};
use core::ecs::{Component, DenseVecStorage, HashMapStorage, VecStorage};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct CompositeRenderable(pub Renderable<'static>);

impl Component for CompositeRenderable {
    type Storage = DenseVecStorage<Self>;
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
    translation: Vec2,
    rotation: Scalar,
    scale: Vec2,
    cached: Mat2d,
}

impl Component for CompositeTransform {
    type Storage = DenseVecStorage<Self>;
}

impl Default for CompositeTransform {
    fn default() -> Self {
        Self {
            translation: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
            cached: Default::default(),
        }
    }
}

impl CompositeTransform {
    pub fn new(translation: Vec2, rotation: Scalar, scale: Vec2) -> Self {
        let mut result = Self {
            translation,
            rotation,
            scale,
            cached: Default::default(),
        };
        result.rebuild();
        result
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
        self.rebuild();
        self
    }

    pub fn with_rotation(mut self, v: Scalar) -> Self {
        self.rotation = v;
        self.rebuild();
        self
    }

    pub fn with_scale(mut self, v: Vec2) -> Self {
        self.scale = v;
        self.rebuild();
        self
    }

    pub fn get_translation(&self) -> Vec2 {
        self.translation
    }

    pub fn get_rotation(&self) -> Scalar {
        self.rotation
    }

    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_translation(&mut self, v: Vec2) {
        self.translation = v;
        self.rebuild();
    }

    pub fn set_rotation(&mut self, v: Scalar) {
        self.rotation = v;
        self.rebuild();
    }

    pub fn set_scale(&mut self, v: Vec2) {
        self.scale = v;
        self.rebuild();
    }

    pub fn matrix(&self) -> Mat2d {
        self.cached.clone()
    }

    fn rebuild(&mut self) {
        let t = Mat2d::translation(self.translation);
        let r = Mat2d::rotation(self.rotation);
        let s = Mat2d::scale(self.scale);
        self.cached = t * r * s;
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct CompositeRenderDepth(pub Scalar);

impl Component for CompositeRenderDepth {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default, Clone)]
pub struct CompositeEffect(pub Effect);

impl Component for CompositeEffect {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub enum CompositeScalingMode {
    None,
    Center,
    Aspect,
    CenterAspect,
}

impl Default for CompositeScalingMode {
    fn default() -> Self {
        CompositeScalingMode::None
    }
}

#[derive(Debug, Clone)]
pub struct CompositeCamera {
    pub scaling: CompositeScalingMode,
    pub tags: Vec<Cow<'static, str>>,
}

impl Component for CompositeCamera {
    type Storage = HashMapStorage<Self>;
}

impl Default for CompositeCamera {
    fn default() -> Self {
        Self {
            scaling: CompositeScalingMode::None,
            tags: vec![],
        }
    }
}

impl CompositeCamera {
    pub fn new(scaling: CompositeScalingMode) -> Self {
        Self {
            scaling,
            tags: vec![],
        }
    }

    pub fn view_matrix(&self, transform: &CompositeTransform, screen_size: Vec2) -> Mat2d {
        let wh = screen_size.x * 0.5;
        let hh = screen_size.y * 0.5;
        let scale = if screen_size.x > screen_size.y {
            screen_size.y
        } else {
            screen_size.x
        };
        let s = Mat2d::scale(Vec2::one() / transform.get_scale());
        let ss = Mat2d::scale(Vec2::new(scale, scale) / transform.get_scale());
        let r = Mat2d::rotation(-transform.get_rotation());
        let t = Mat2d::translation(-transform.get_translation());
        let tt = Mat2d::translation([wh, hh].into());
        match self.scaling {
            CompositeScalingMode::None => s * r * t,
            CompositeScalingMode::Center => tt * s * r * t,
            CompositeScalingMode::Aspect => ss * r * t,
            CompositeScalingMode::CenterAspect => tt * ss * r * t,
        }
    }
}
