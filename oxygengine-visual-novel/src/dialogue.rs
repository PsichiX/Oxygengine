use anim::transition::{SwitchTransition, Transition};
use core::{prefab::Prefab, Scalar};
use serde::{Deserialize, Serialize};

pub type ActiveDialogue = Transition<Option<Dialogue>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub character: String,
    pub text: String,
    #[serde(default)]
    pub options: Vec<DialogueOption>,
}

impl Prefab for Dialogue {}

impl Dialogue {
    pub fn is_dirty(&self) -> bool {
        self.options.iter().any(|option| option.is_dirty())
    }

    pub fn process(&mut self, delta_time: Scalar) {
        for option in &mut self.options {
            option.process(delta_time);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    pub action: DialogueAction,
    #[serde(default)]
    pub focused: SwitchTransition,
}

impl Prefab for DialogueOption {}

impl DialogueOption {
    pub fn is_dirty(&self) -> bool {
        self.focused.in_progress()
    }

    pub fn process(&mut self, delta_time: Scalar) {
        self.focused.process(delta_time);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueAction {
    None,
    JumpToLabel(String),
    JumpToChapter(String),
}

impl Default for DialogueAction {
    fn default() -> Self {
        Self::None
    }
}

impl Prefab for DialogueAction {}
