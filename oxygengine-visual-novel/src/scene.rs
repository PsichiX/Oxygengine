use crate::{background::BackgroundStyle, Position};
use anim::{animation::Interpolation, transition::Transition};
use core::{prefab::Prefab, Scalar, Ignite};
use serde::{Deserialize, Serialize};

pub type ActiveScene = Transition<Option<String>>;

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Scene {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub background_style: BackgroundStyle,
    #[serde(default)]
    pub camera_position: Interpolation<Position>,
    #[serde(default)]
    pub camera_rotation: Interpolation<Scalar>,
}

impl Prefab for Scene {}

impl Scene {
    pub fn initialize(&mut self) {
        self.background_style.end();
        self.camera_position.end();
        self.camera_rotation.end();
    }

    pub fn in_progress(&self) -> bool {
        self.background_style.in_progress()
            || self.camera_position.in_progress()
            || self.camera_rotation.in_progress()
    }

    pub fn process(&mut self, delta_time: Scalar) {
        self.background_style.process(delta_time);
        self.camera_position.process(delta_time);
        self.camera_rotation.process(delta_time);
    }
}
