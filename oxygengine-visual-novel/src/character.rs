use crate::{Color, Position, Scale};
use anim::{animation::Interpolation, transition::Transition};
use core::{prefab::Prefab, Scalar, Ignite};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type CharacterStyle = Transition<String>;

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    name: String,
    /// {style name: image name}
    pub styles: HashMap<String, String>,
    #[serde(default)]
    style: CharacterStyle,
    #[serde(default)]
    visibility: Interpolation<Scalar>,
    #[serde(default)]
    name_color: Interpolation<Color>,
    /// position in screen space percentage.
    #[serde(default)]
    position: Interpolation<Position>,
    /// alignment tells where in the image space pivot is located (percentage of image size).
    #[serde(default)]
    alignment: Interpolation<Position>,
    #[serde(default)]
    rotation: Interpolation<Scalar>,
    #[serde(default)]
    scale: Interpolation<Scale>,
    #[serde(skip)]
    pub(crate) dirty: bool,
}

impl Prefab for Character {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: Default::default(),
            styles: Default::default(),
            style: Default::default(),
            visibility: Interpolation::instant(1.0),
            name_color: Interpolation::instant((1.0, 1.0, 1.0).into()),
            position: Default::default(),
            alignment: Default::default(),
            rotation: Default::default(),
            scale: Interpolation::instant((1.0, 1.0).into()),
            dirty: true,
        }
    }
}

impl Character {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub fn initialize(&mut self) {
        self.style.end();
        self.visibility.end();
        self.name_color.end();
        self.position.end();
        self.alignment.end();
        self.rotation.end();
        self.scale.end();
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
            || self.style.in_progress()
            || self.visibility.in_progress()
            || self.name_color.in_progress()
            || self.position.in_progress()
            || self.alignment.in_progress()
            || self.rotation.in_progress()
            || self.scale.in_progress()
    }

    pub fn rebuild(&mut self) {
        self.dirty = true;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
        self.dirty = true;
    }

    /// (style name, image name)
    pub fn styles(&self) -> impl Iterator<Item = (&str, &str)> {
        self.styles.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// (from style, phase, to style)
    pub fn style(&self) -> (&str, Scalar, &str) {
        let from = self.style.from().as_str();
        let to = self.style.to().as_str();
        let phase = self.style.phase();
        (from, phase, to)
    }

    pub fn style_transition(&self) -> &CharacterStyle {
        &self.style
    }

    pub fn style_transition_mut(&mut self) -> &mut CharacterStyle {
        &mut self.style
    }

    pub fn set_style(&mut self, value: String) {
        self.style.set(value);
        self.style.playing = true;
    }

    pub fn visibility(&self) -> Scalar {
        self.visibility.value()
    }

    pub fn visibility_anim(&self) -> &Interpolation<Scalar> {
        &self.visibility
    }

    pub fn visibility_anim_mut(&mut self) -> &mut Interpolation<Scalar> {
        &mut self.visibility
    }

    pub fn set_visibility(&mut self, value: Scalar) {
        self.visibility.set(value);
        self.visibility.playing = true;
    }

    pub fn hide(&mut self) {
        self.set_visibility(0.0);
    }

    pub fn show(&mut self) {
        self.set_visibility(1.0);
    }

    pub fn name_color(&self) -> Color {
        self.name_color.value()
    }

    pub fn name_color_anim(&self) -> &Interpolation<Color> {
        &self.name_color
    }

    pub fn name_color_anim_mut(&mut self) -> &mut Interpolation<Color> {
        &mut self.name_color
    }

    pub fn set_name_color(&mut self, value: Color) {
        self.name_color.set(value);
        self.name_color.playing = true;
    }

    pub fn position(&self) -> Position {
        self.position.value()
    }

    pub fn position_anim(&self) -> &Interpolation<Position> {
        &self.position
    }

    pub fn position_anim_mut(&mut self) -> &mut Interpolation<Position> {
        &mut self.position
    }

    pub fn set_position(&mut self, value: Position) {
        self.position.set(value);
        self.position.playing = true;
    }

    pub fn alignment(&self) -> Position {
        self.alignment.value()
    }

    pub fn alignment_anim(&self) -> &Interpolation<Position> {
        &self.alignment
    }

    pub fn alignment_anim_mut(&mut self) -> &mut Interpolation<Position> {
        &mut self.alignment
    }

    pub fn set_alignment(&mut self, value: Position) {
        self.alignment.set(value);
        self.alignment.playing = true;
    }

    pub fn rotation(&self) -> Scalar {
        self.rotation.value()
    }

    pub fn rotation_anim(&self) -> &Interpolation<Scalar> {
        &self.rotation
    }

    pub fn rotation_anim_mut(&mut self) -> &mut Interpolation<Scalar> {
        &mut self.rotation
    }

    pub fn set_rotation(&mut self, value: Scalar) {
        self.rotation.set(value);
        self.rotation.playing = true;
    }

    pub fn scale(&self) -> Scale {
        self.scale.value()
    }

    pub fn scale_anim(&self) -> &Interpolation<Scale> {
        &self.scale
    }

    pub fn scale_anim_mut(&mut self) -> &mut Interpolation<Scale> {
        &mut self.scale
    }

    pub fn set_scale(&mut self, value: Scale) {
        self.scale.set(value);
        self.scale.playing = true;
    }

    pub fn process(&mut self, delta_time: Scalar) {
        self.dirty = false;
        self.style.process(delta_time);
        self.visibility.process(delta_time);
        self.name_color.process(delta_time);
        self.position.process(delta_time);
        self.alignment.process(delta_time);
        self.rotation.process(delta_time);
        self.scale.process(delta_time);
    }
}
