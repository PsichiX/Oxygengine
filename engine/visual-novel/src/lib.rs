extern crate oxygengine_animation as anim;
extern crate oxygengine_core as core;

pub mod background;
pub mod character;
pub mod dialogue;
pub mod resource;
pub mod scene;
pub mod script;
pub mod story;
pub mod system;
pub mod vn_story_asset_protocol;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::background::*;
    pub use crate::character::*;
    pub use crate::dialogue::*;
    pub use crate::resource::*;
    pub use crate::scene::*;
    pub use crate::script::*;
    pub use crate::story::*;
    pub use crate::system::*;
    pub use crate::vn_story_asset_protocol::*;
}

use crate::{
    resource::VnStoryManager,
    system::{vn_story_system, VnStorySystemCache, VnStorySystemResources},
};
use anim::curve::{Curved, CurvedChange};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

pub type Scale = Position;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Position(pub Scalar, pub Scalar);

impl Curved for Position {
    fn zero() -> Self {
        Self(0.0, 0.0)
    }

    fn one() -> Self {
        Self(1.0, 1.0)
    }

    fn negate(&self) -> Self {
        Self(-self.0, -self.1)
    }

    fn scale(&self, value: Scalar) -> Self {
        Self(self.0 * value, self.1 * value)
    }

    fn inverse_scale(&self, value: Scalar) -> Self {
        Self(self.0 / value, self.1 / value)
    }

    fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    fn length_squared(&self) -> Scalar {
        self.0 * self.0 + self.1 * self.1
    }

    fn get_axis(&self, index: usize) -> Option<Scalar> {
        match index {
            0 => Some(self.0),
            1 => Some(self.1),
            _ => None,
        }
    }

    fn interpolate(&self, other: &Self, factor: Scalar) -> Self {
        let diff = *other - *self;
        diff * factor + *self
    }

    fn is_valid(&self) -> bool {
        self.0.is_valid() && self.1.is_valid()
    }
}

impl CurvedChange for Position {
    fn offset(&self, other: &Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }

    fn delta(&self, other: &Self) -> Self {
        Self(other.0 - self.0, other.1 - self.1)
    }

    fn dot(&self, other: &Self) -> Scalar {
        self.0 * other.0 + self.1 * other.1
    }
}

impl Add<Self> for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub<Self> for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl Mul<Scalar> for Position {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Self(self.0 * other, self.1 * other)
    }
}

impl From<(Scalar, Scalar)> for Position {
    fn from(value: (Scalar, Scalar)) -> Self {
        Self(value.0, value.1)
    }
}

impl From<[Scalar; 2]> for Position {
    fn from(value: [Scalar; 2]) -> Self {
        Self(value[0], value[1])
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Color(pub Scalar, pub Scalar, pub Scalar);

impl Curved for Color {
    fn zero() -> Self {
        Self(0.0, 0.0, 0.0)
    }

    fn one() -> Self {
        Self(1.0, 1.0, 1.0)
    }

    fn negate(&self) -> Self {
        Self(-self.0, -self.1, -self.2)
    }

    fn scale(&self, value: Scalar) -> Self {
        Self(self.0 * value, self.1 * value, self.2 * value)
    }

    fn inverse_scale(&self, value: Scalar) -> Self {
        Self(self.0 / value, self.1 / value, self.2 / value)
    }

    fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    fn length_squared(&self) -> Scalar {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    fn get_axis(&self, index: usize) -> Option<Scalar> {
        match index {
            0 => Some(self.0),
            1 => Some(self.1),
            2 => Some(self.2),
            _ => None,
        }
    }

    fn interpolate(&self, other: &Self, factor: Scalar) -> Self {
        let diff = *other - *self;
        diff * factor + *self
    }

    fn is_valid(&self) -> bool {
        self.0.is_valid() && self.1.is_valid() && self.2.is_valid()
    }
}

impl CurvedChange for Color {
    fn offset(&self, other: &Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }

    fn delta(&self, other: &Self) -> Self {
        Self(other.0 - self.0, other.1 - self.1, other.2 - self.2)
    }

    fn dot(&self, other: &Self) -> Scalar {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }
}

impl Add<Self> for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Sub<Self> for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl Mul<Scalar> for Color {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Self(self.0 * other, self.1 * other, self.2 * other)
    }
}

impl From<(Scalar, Scalar, Scalar)> for Color {
    fn from(value: (Scalar, Scalar, Scalar)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

impl From<[Scalar; 3]> for Color {
    fn from(value: [Scalar; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(VnStoryManager::default());
    builder.install_resource(VnStorySystemCache::default());
    builder.install_system::<VnStorySystemResources>("vn-story", vn_story_system, &[])?;
    Ok(())
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(vn_story_asset_protocol::VnStoryAssetProtocol);
}
