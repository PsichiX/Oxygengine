use anim::transition::Transition;
use core::prefab::Prefab;
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    pub action: DialogueAction,
}

impl Prefab for DialogueOption {}

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
