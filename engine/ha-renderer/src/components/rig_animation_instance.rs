use crate::{asset_protocols::rig_animation::*, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct ActiveSequence {
    pub blend_weight: Scalar,
    pub time: Scalar,
    pub change_time: Scalar,
    pub change_time_limit: Scalar,
    pub bounced: bool,
}

impl ActiveSequence {
    pub fn change_weight(&self) -> Scalar {
        if self.change_time_limit > 0.0 {
            self.change_time / self.change_time_limit
        } else {
            1.0
        }
    }

    pub fn weight(&self) -> Scalar {
        self.blend_weight * self.change_weight()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Active {
    pub state: String,
    pub current_sequences: HashMap<String, ActiveSequence>,
    pub old_sequences: HashMap<String, ActiveSequence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaRigAnimationInstance {
    #[serde(default)]
    pub playing: bool,
    #[serde(default = "HaRigAnimationInstance::default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub values: HashMap<String, RigAnimationValue>,
    #[serde(default)]
    pub(crate) animation: String,
    #[serde(skip)]
    pub(crate) active: Option<Active>,
    #[serde(skip)]
    pub(crate) signals: Vec<RigAnimationSignal>,
}

impl Default for HaRigAnimationInstance {
    fn default() -> Self {
        Self {
            playing: false,
            speed: Self::default_speed(),
            values: Default::default(),
            animation: Default::default(),
            active: None,
            signals: Default::default(),
        }
    }
}

impl HaRigAnimationInstance {
    fn default_speed() -> Scalar {
        1.0
    }

    pub fn active_state(&self) -> Option<&str> {
        self.active.as_ref().map(|a| a.state.as_str())
    }

    /// (name, time)?
    pub fn active_current_sequences(&self) -> Option<impl Iterator<Item = (&str, Scalar)>> {
        self.active.as_ref().map(|a| {
            a.current_sequences
                .iter()
                .map(|(n, s)| (n.as_str(), s.time))
        })
    }

    /// [(name, time)]?
    pub fn active_old_sequences(&self) -> Option<impl Iterator<Item = (&str, Scalar)>> {
        self.active
            .as_ref()
            .map(|a| a.old_sequences.iter().map(|(n, s)| (n.as_str(), s.time)))
    }

    pub fn received_signals(&self) -> &[RigAnimationSignal] {
        &self.signals
    }

    pub fn play(&mut self, state: impl ToString) {
        self.playing = true;
        self.active = Some(Active {
            state: state.to_string(),
            current_sequences: Default::default(),
            old_sequences: Default::default(),
        });
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.active = None;
    }

    pub fn animation(&self) -> &str {
        &self.animation
    }

    pub fn set_animation(&mut self, animation: impl ToString) {
        self.animation = animation.to_string();
    }

    pub fn set_value(&mut self, name: impl ToString, value: impl Into<RigAnimationValue>) {
        self.values.insert(name.to_string(), value.into());
    }

    pub fn unset_value(&mut self, name: &str) -> Option<RigAnimationValue> {
        self.values.remove(name)
    }
}

impl Prefab for HaRigAnimationInstance {}
impl PrefabComponent for HaRigAnimationInstance {}
