use crate::math::*;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

fn default_speed() -> Scalar {
    1.0
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteAnimationRegion {
    None,
    All,
    From(usize),
    To(usize),
    /// (from inclusive, to exclusive)
    Range(usize, usize),
}

impl Default for SpriteAnimationRegion {
    fn default() -> Self {
        Self::All
    }
}

impl SpriteAnimationRegion {
    pub fn contains(self, position: usize) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            Self::From(index) => position >= index,
            Self::To(index) => position < index,
            Self::Range(from, to) => position >= from && position < to,
        }
    }
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpriteAnimationValue {
    Bool(bool),
    Integer(i32),
    Scalar(Scalar),
}

impl SpriteAnimationValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_scalar(&self) -> Option<Scalar> {
        match self {
            Self::Scalar(v) => Some(*v),
            _ => None,
        }
    }
}

impl From<bool> for SpriteAnimationValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i32> for SpriteAnimationValue {
    fn from(v: i32) -> Self {
        Self::Integer(v)
    }
}

impl From<Scalar> for SpriteAnimationValue {
    fn from(v: Scalar) -> Self {
        Self::Scalar(v)
    }
}

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SpriteAnimationCondition {
    Bool(bool),
    IntegerEquals(i32),
    IntegerNotEquals(i32),
    IntegerGreater(i32),
    IntegerLess(i32),
    /// (from inclusive, to inclusive)
    IntegerRange(i32, i32),
    /// (value, threshold)
    ScalarNearlyEquals(Scalar, Scalar),
    /// (value, threshold)
    ScalarNotNearlyEquals(Scalar, Scalar),
    ScalarGreater(Scalar),
    ScalarLess(Scalar),
    /// (from inclusive, to inclusive)
    ScalarRange(Scalar, Scalar),
}

impl SpriteAnimationCondition {
    pub fn validate(&self, value: &SpriteAnimationValue) -> bool {
        match (self, value) {
            (Self::Bool(c), SpriteAnimationValue::Bool(v)) => c == v,
            (Self::IntegerEquals(c), SpriteAnimationValue::Integer(v)) => c == v,
            (Self::IntegerNotEquals(c), SpriteAnimationValue::Integer(v)) => c != v,
            (Self::IntegerGreater(c), SpriteAnimationValue::Integer(v)) => v > c,
            (Self::IntegerLess(c), SpriteAnimationValue::Integer(v)) => v < c,
            (Self::IntegerRange(a, b), SpriteAnimationValue::Integer(v)) => v >= a && v <= b,
            (Self::ScalarNearlyEquals(c, t), SpriteAnimationValue::Scalar(v)) => (c - v).abs() < *t,
            (Self::ScalarNotNearlyEquals(c, t), SpriteAnimationValue::Scalar(v)) => {
                (c - v).abs() >= *t
            }
            (Self::ScalarGreater(c), SpriteAnimationValue::Scalar(v)) => v > c,
            (Self::ScalarLess(c), SpriteAnimationValue::Scalar(v)) => v < c,
            (Self::ScalarRange(a, b), SpriteAnimationValue::Scalar(v)) => v >= a && v <= b,
            _ => false,
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimationBlendState {
    pub target_state: String,
    pub axis_values: Vec<Scalar>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SpriteAnimationRule {
    Single {
        target_state: String,
        #[serde(default)]
        conditions: HashMap<String, SpriteAnimationCondition>,
        #[serde(default)]
        region: SpriteAnimationRegion,
    },
    BlendSpace {
        axis_scalars: Vec<String>,
        blend_states: Vec<SpriteAnimationBlendState>,
        #[serde(default)]
        conditions: HashMap<String, SpriteAnimationCondition>,
    },
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpriteAnimationSignal {
    pub time: Scalar,
    pub id: String,
    #[serde(default)]
    pub params: HashMap<String, SpriteAnimationValue>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpriteAnimationState {
    pub frames: Vec<String>,
    #[serde(default)]
    pub signals: Vec<SpriteAnimationSignal>,
    #[serde(default = "default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub bounce: bool,
    #[serde(default)]
    pub rules: Vec<SpriteAnimationRule>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpriteAnimationAsset {
    #[serde(default)]
    pub default_state: Option<String>,
    #[serde(default = "default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub states: HashMap<String, SpriteAnimationState>,
    #[serde(default)]
    pub rules: Vec<SpriteAnimationRule>,
}

pub struct SpriteAnimationAssetProtocol;

impl AssetProtocol for SpriteAnimationAssetProtocol {
    fn name(&self) -> &str {
        "spriteanim"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<SpriteAnimationAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<SpriteAnimationAsset>(data).unwrap()
        } else {
            bincode::deserialize::<SpriteAnimationAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(data))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
