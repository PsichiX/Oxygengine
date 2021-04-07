use crate::ui::{
    new_theme,
    screens::{hud::hud, menu::menu, notifications::notifications},
};
use oxygengine::user_interface::raui::core::prelude::*;

widget_component! {
    pub gui(key, named_slots) {
        widget! {
            (#{key} content_box | {new_theme()} [
                (#{"hud"} hud)
                (#{"menu"} menu)
                (#{"notifications"} notifications)
            ])
        }
    }
}
