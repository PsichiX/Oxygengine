pub mod screens;

use oxygengine::user_interface::raui::core::prelude::*;

pub fn setup(app: &mut Application) {
    app.register_component("intro", screens::intro::intro);
    app.register_component("loading", screens::loading::loading);
    app.register_component("menu", screens::menu::menu);
    app.register_component("gui", screens::gui::gui);
}
