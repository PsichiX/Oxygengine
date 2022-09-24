use crate::{components::transform::HaTransform, math::*};
use animation::phase::Phase;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Range, str::from_utf8};

fn default_speed() -> Scalar {
    1.0
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkeletalAnimationValue {
    Bool(bool),
    Integer(i32),
    Scalar(Scalar),
    String(String),
}

impl SkeletalAnimationValue {
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

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v.as_str()),
            _ => None,
        }
    }
}

impl From<bool> for SkeletalAnimationValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i32> for SkeletalAnimationValue {
    fn from(v: i32) -> Self {
        Self::Integer(v)
    }
}

impl From<Scalar> for SkeletalAnimationValue {
    fn from(v: Scalar) -> Self {
        Self::Scalar(v)
    }
}

impl From<String> for SkeletalAnimationValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SkeletalAnimationCondition {
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
    StringEquals(String),
    StringNotEquals(String),
    StringContains(String),
    StringNotContains(String),
    StringTagged {
        tag: String,
        separator: String,
    },
    StringNotTagged {
        tag: String,
        separator: String,
    },
}

impl SkeletalAnimationCondition {
    pub fn validate(&self, value: &SkeletalAnimationValue) -> bool {
        match (self, value) {
            (Self::Bool(c), SkeletalAnimationValue::Bool(v)) => c == v,
            (Self::IntegerEquals(c), SkeletalAnimationValue::Integer(v)) => c == v,
            (Self::IntegerNotEquals(c), SkeletalAnimationValue::Integer(v)) => c != v,
            (Self::IntegerGreater(c), SkeletalAnimationValue::Integer(v)) => v > c,
            (Self::IntegerLess(c), SkeletalAnimationValue::Integer(v)) => v < c,
            (Self::IntegerRange(a, b), SkeletalAnimationValue::Integer(v)) => v >= a && v <= b,
            (Self::ScalarNearlyEquals(c, t), SkeletalAnimationValue::Scalar(v)) => {
                (c - v).abs() < *t
            }
            (Self::ScalarNotNearlyEquals(c, t), SkeletalAnimationValue::Scalar(v)) => {
                (c - v).abs() >= *t
            }
            (Self::ScalarGreater(c), SkeletalAnimationValue::Scalar(v)) => v > c,
            (Self::ScalarLess(c), SkeletalAnimationValue::Scalar(v)) => v < c,
            (Self::ScalarRange(a, b), SkeletalAnimationValue::Scalar(v)) => v >= a && v <= b,
            (Self::StringEquals(c), SkeletalAnimationValue::String(v)) => c == v,
            (Self::StringNotEquals(c), SkeletalAnimationValue::String(v)) => c != v,
            (Self::StringContains(c), SkeletalAnimationValue::String(v)) => v.contains(c),
            (Self::StringNotContains(c), SkeletalAnimationValue::String(v)) => !v.contains(c),
            (Self::StringTagged { tag, separator }, SkeletalAnimationValue::String(v)) => {
                v.split(separator).any(|part| part == tag)
            }
            (Self::StringNotTagged { tag, separator }, SkeletalAnimationValue::String(v)) => {
                !v.split(separator).any(|part| part == tag)
            }
            _ => false,
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationBlendState {
    pub target_state: String,
    pub axis_values: Vec<Scalar>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SkeletalAnimationRule {
    Single {
        target_state: String,
        #[serde(default)]
        conditions: HashMap<String, SkeletalAnimationCondition>,
        #[serde(default)]
        change_time: Scalar,
    },
    BlendSpace {
        axis_scalars: Vec<String>,
        blend_states: Vec<SkeletalAnimationBlendState>,
        #[serde(default)]
        conditions: HashMap<String, SkeletalAnimationCondition>,
        #[serde(default)]
        change_time: Scalar,
    },
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationSignal {
    pub time: Scalar,
    pub id: String,
    #[serde(default)]
    pub params: HashMap<String, SkeletalAnimationValue>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationSequenceBoneSheet {
    #[serde(default)]
    pub translation_x: Option<Phase>,
    #[serde(default)]
    pub translation_y: Option<Phase>,
    #[serde(default)]
    pub translation_z: Option<Phase>,
    #[serde(default)]
    pub rotation_yaw: Option<Phase>,
    #[serde(default)]
    pub rotation_pitch: Option<Phase>,
    #[serde(default)]
    pub rotation_roll: Option<Phase>,
    #[serde(default)]
    pub scale_x: Option<Phase>,
    #[serde(default)]
    pub scale_y: Option<Phase>,
    #[serde(default)]
    pub scale_z: Option<Phase>,
}

impl SkeletalAnimationSequenceBoneSheet {
    pub fn time_frame(&self) -> Option<Range<Scalar>> {
        let mut result = None;
        Self::accumulate_time_frame(&self.translation_x, &mut result);
        Self::accumulate_time_frame(&self.translation_y, &mut result);
        Self::accumulate_time_frame(&self.translation_z, &mut result);
        Self::accumulate_time_frame(&self.rotation_yaw, &mut result);
        Self::accumulate_time_frame(&self.rotation_pitch, &mut result);
        Self::accumulate_time_frame(&self.rotation_roll, &mut result);
        Self::accumulate_time_frame(&self.scale_x, &mut result);
        Self::accumulate_time_frame(&self.scale_y, &mut result);
        Self::accumulate_time_frame(&self.scale_z, &mut result);
        result
    }

    pub fn sample(&self, time: Scalar, fallback: &HaTransform) -> HaTransform {
        HaTransform::new(
            vec3(
                self.translation_x
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_translation().x),
                self.translation_y
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_translation().y),
                self.translation_z
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_translation().z),
            ),
            Eulers {
                yaw: self
                    .rotation_yaw
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_rotation().eulers().yaw),
                pitch: self
                    .rotation_pitch
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_rotation().eulers().pitch),
                roll: self
                    .rotation_roll
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_rotation().eulers().roll),
            },
            vec3(
                self.scale_x
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_scale().x),
                self.scale_y
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_scale().y),
                self.scale_z
                    .as_ref()
                    .map(|phase| phase.sample(time))
                    .unwrap_or_else(|| fallback.get_scale().z),
            ),
        )
    }

    fn accumulate_time_frame(input: &Option<Phase>, output: &mut Option<Range<Scalar>>) {
        if let Some(phase) = input {
            let time_frame = phase.time_frame();
            if let Some(output) = output.as_mut() {
                output.start = output.start.min(time_frame.start);
                output.end = output.end.min(time_frame.end);
            } else {
                *output = Some(time_frame);
            }
        }
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationSequence {
    #[serde(default = "default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub bounce: bool,
    #[serde(default)]
    pub bone_sheets: HashMap<String, SkeletalAnimationSequenceBoneSheet>,
    #[serde(default)]
    pub signals: Vec<SkeletalAnimationSignal>,
}

impl SkeletalAnimationSequence {
    pub fn time_frame(&self) -> Option<Range<Scalar>> {
        self.bone_sheets
            .values()
            .fold(None, |a, v| match (a, v.time_frame()) {
                (None, None) => None,
                (Some(a), None) => Some(a),
                (Some(a), Some(v)) => Some(a.start.min(v.start)..a.end.max(v.end)),
                (None, Some(v)) => Some(v),
            })
    }

    pub fn sample_bone(&self, name: &str, time: Scalar, fallback: &HaTransform) -> HaTransform {
        self.bone_sheets
            .get(name)
            .map(|sheet| sheet.sample(time, fallback))
            .unwrap_or_else(|| fallback.to_owned())
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationBlendSpace {
    pub sequence: String,
    pub axis_values: Vec<Scalar>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SkeletalAnimationStateSequences {
    Single(String),
    BlendSpace {
        axis_scalars: Vec<String>,
        sequences: Vec<SkeletalAnimationBlendSpace>,
    },
}

impl Default for SkeletalAnimationStateSequences {
    fn default() -> Self {
        Self::Single(Default::default())
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationState {
    pub sequences: SkeletalAnimationStateSequences,
    #[serde(default)]
    pub rules: Vec<SkeletalAnimationRule>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimationAsset {
    #[serde(default)]
    pub default_state: Option<String>,
    #[serde(default = "default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub sequences: HashMap<String, SkeletalAnimationSequence>,
    #[serde(default)]
    pub states: HashMap<String, SkeletalAnimationState>,
    #[serde(default)]
    pub rules: Vec<SkeletalAnimationRule>,
}

pub struct SkeletalAnimationAssetProtocol;

impl AssetProtocol for SkeletalAnimationAssetProtocol {
    fn name(&self) -> &str {
        "skelanim"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let data = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<SkeletalAnimationAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<SkeletalAnimationAsset>(data).unwrap()
        } else {
            bincode::deserialize::<SkeletalAnimationAsset>(&data).unwrap()
        };
        AssetLoadResult::Data(Box::new(data))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
