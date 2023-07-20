pub mod components;
pub mod gui;

use oxygengine::user_interface::raui::core::prelude::*;

pub fn setup(app: &mut Application) {
    app.register_component("gui", gui::gui);
}
