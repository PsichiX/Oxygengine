use oxygengine::user_interface::raui::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MenuState {
    pub opened: bool,
}
