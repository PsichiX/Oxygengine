use oxygengine_core::Scalar;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

fn default_scale() -> Scalar {
    1.0
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skeleton {
    pub hash: String,
    pub spine: String,
    pub x: Scalar,
    pub y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
    #[serde(default)]
    pub images: PathBuf,
    #[serde(default)]
    pub audio: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformMode {
    Normal,
    OnlyTranslation,
    NoRotationOrReflection,
    NoScale,
    NoScaleOrReflection,
}

impl Default for TransformMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bone {
    pub name: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub length: Scalar,
    #[serde(default)]
    pub transform: TransformMode,
    #[serde(default)]
    pub x: Scalar,
    #[serde(default)]
    pub y: Scalar,
    #[serde(default)]
    pub rotation: Scalar,
    #[serde(default = "default_scale")]
    pub scale_x: Scalar,
    #[serde(default = "default_scale")]
    pub scale_y: Scalar,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SlotBlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
}

impl Default for SlotBlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub name: String,
    pub bone: String,
    #[serde(default)]
    pub attachment: Option<String>,
    #[serde(default)]
    pub blend: SlotBlendMode,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinAttachmentRegion {
    #[serde(default)]
    pub x: Scalar,
    #[serde(default)]
    pub y: Scalar,
    #[serde(default = "default_scale")]
    pub scale_x: Scalar,
    #[serde(default = "default_scale")]
    pub scale_y: Scalar,
    #[serde(default)]
    pub rotation: Scalar,
    pub width: usize,
    pub height: usize,
}

pub type SkinAttachmentSlot = HashMap<String, SkinAttachmentRegion>;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub name: String,
    #[serde(default)]
    pub attachments: HashMap<String, SkinAttachmentSlot>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "int")]
    #[serde(default)]
    pub int_value: isize,
    #[serde(rename = "float")]
    #[serde(default)]
    pub float_value: Scalar,
    #[serde(rename = "string")]
    #[serde(default)]
    pub string_value: Option<String>,
    #[serde(default)]
    pub audio: Option<PathBuf>,
    #[serde(default)]
    pub volume: Scalar,
    #[serde(default)]
    pub balance: Scalar,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum AnimationCurve {
    Curve(Vec<Scalar>),
    Custom(String),
    #[serde(skip)]
    Linear,
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationBoneRotate {
    #[serde(default)]
    pub time: Scalar,
    #[serde(default)]
    #[serde(alias = "angle")]
    pub value: Scalar,
    #[serde(default)]
    pub curve: AnimationCurve,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationBoneTranslateOrScale {
    #[serde(default)]
    pub time: Scalar,
    #[serde(default)]
    pub x: Scalar,
    #[serde(default)]
    pub y: Scalar,
    #[serde(default)]
    pub curve: AnimationCurve,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationBone {
    #[serde(default)]
    pub rotate: Vec<AnimationBoneRotate>,
    #[serde(default)]
    pub translate: Vec<AnimationBoneTranslateOrScale>,
    #[serde(default)]
    pub scale: Vec<AnimationBoneTranslateOrScale>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationEvent {
    #[serde(default)]
    pub time: Scalar,
    pub name: String,
    #[serde(rename = "int")]
    #[serde(default)]
    pub int_value: Option<isize>,
    #[serde(rename = "float")]
    #[serde(default)]
    pub float_value: Option<Scalar>,
    #[serde(rename = "string")]
    #[serde(default)]
    pub string_value: Option<String>,
    #[serde(default)]
    pub volume: Option<Scalar>,
    #[serde(default)]
    pub balance: Option<Scalar>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    #[serde(default)]
    pub bones: HashMap<String, AnimationBone>,
    #[serde(default)]
    pub events: Vec<AnimationEvent>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub skeleton: Skeleton,
    #[serde(default)]
    pub bones: Vec<Bone>,
    #[serde(default)]
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub skins: Vec<Skin>,
    #[serde(default)]
    pub events: HashMap<String, Event>,
    #[serde(default)]
    pub animations: HashMap<String, Animation>,
}
