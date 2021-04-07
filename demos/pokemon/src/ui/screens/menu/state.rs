use oxygengine::user_interface::raui::core::implement_props_data;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuState {
    pub opened: bool,
}
implement_props_data!(MenuState);

impl Default for MenuState {
    fn default() -> Self {
        Self { opened: false }
    }
}
