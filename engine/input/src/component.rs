use crate::resources::stack::*;
use core::prefab::{Prefab, PrefabComponent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputStackInstance {
    Setup(InputStackListener),
    #[serde(skip)]
    Listener(InputStackListenerId),
}

impl Default for InputStackInstance {
    fn default() -> Self {
        Self::Setup(Default::default())
    }
}

impl InputStackInstance {
    pub fn as_listener(&self) -> Option<InputStackListenerId> {
        match self {
            Self::Listener(id) => Some(*id),
            _ => None,
        }
    }
}

impl Prefab for InputStackInstance {}
impl PrefabComponent for InputStackInstance {}
