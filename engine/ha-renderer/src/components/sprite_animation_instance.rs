use crate::{asset_protocols::sprite_animation::*, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct Active {
    pub state: String,
    pub frame: Scalar,
    pub bounced: bool,
    pub cached_frame: Option<String>,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaSpriteAnimationInstance {
    #[serde(default)]
    pub playing: bool,
    #[serde(default = "HaSpriteAnimationInstance::default_speed")]
    pub speed: Scalar,
    #[serde(default)]
    pub values: HashMap<String, SpriteAnimationValue>,
    #[serde(default)]
    pub(crate) animation: String,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) active: Option<Active>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) frame_changed: bool,
}

impl Default for HaSpriteAnimationInstance {
    fn default() -> Self {
        Self {
            playing: false,
            speed: Self::default_speed(),
            values: Default::default(),
            animation: Default::default(),
            active: None,
            frame_changed: false,
        }
    }
}

impl HaSpriteAnimationInstance {
    fn default_speed() -> Scalar {
        1.0
    }

    /// (state name, fractional frame)
    pub fn active_state(&self) -> Option<(&str, Scalar)> {
        self.active.as_ref().map(|a| (a.state.as_str(), a.frame))
    }

    pub fn active_frame(&self) -> Option<&str> {
        if let Some(active) = &self.active {
            return active.cached_frame.as_deref();
        }
        None
    }

    pub fn frame_lately_changed(&self) -> bool {
        self.frame_changed
    }

    pub fn play(&mut self, state: impl ToString) {
        self.playing = true;
        self.active = Some(Active {
            state: state.to_string(),
            frame: 0.0,
            bounced: false,
            cached_frame: None,
        });
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.active = None;
        self.frame_changed = true;
    }

    pub fn animation(&self) -> &str {
        &self.animation
    }

    pub fn set_animation(&mut self, animation: impl ToString) {
        self.animation = animation.to_string();
    }

    pub fn set_value(&mut self, name: impl ToString, value: SpriteAnimationValue) {
        self.values.insert(name.to_string(), value);
    }

    pub fn unset_value(&mut self, name: &str) -> Option<SpriteAnimationValue> {
        self.values.remove(name)
    }
}

impl Prefab for HaSpriteAnimationInstance {}
impl PrefabComponent for HaSpriteAnimationInstance {}
