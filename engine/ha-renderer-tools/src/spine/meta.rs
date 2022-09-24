use oxygengine_core::Scalar;
use oxygengine_ha_renderer::asset_protocols::skeletal_animation::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AnimationMetaStatePerSequence {
    #[serde(default = "AnimationMetaStatePerSequence::default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub change_time: Scalar,
    #[serde(default = "AnimationMetaStatePerSequence::default_value_name")]
    pub value_name: String,
}

impl AnimationMetaStatePerSequence {
    fn default_prefix() -> String {
        "#".to_owned()
    }

    fn default_value_name() -> String {
        "@force-sequence".to_owned()
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AnimationMeta {
    #[serde(default)]
    pub loop_sequences: Vec<String>,
    #[serde(default)]
    pub bounce_sequences: Vec<String>,
    #[serde(default)]
    pub skin: Option<String>,
    #[serde(default)]
    pub states: HashMap<String, SkeletalAnimationState>,
    #[serde(default)]
    pub default_state: Option<String>,
    #[serde(default)]
    pub rules: Vec<SkeletalAnimationRule>,
    #[serde(default)]
    pub make_state_per_sequence: Option<AnimationMetaStatePerSequence>,
}
