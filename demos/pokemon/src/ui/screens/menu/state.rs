use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct MenuState {
    pub opened: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self { opened: false }
    }
}
