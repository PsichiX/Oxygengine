use crate::ui::{
    new_theme,
    screens::{hud::hud, menu::menu, notifications::notifications},
};
use oxygengine::user_interface::raui::core::prelude::*;

pub fn gui(context: WidgetContext) -> WidgetNode {
    widget! {
        (#{context.key} content_box | {new_theme()} [
            (#{"hud"} hud)
            (#{"menu"} menu)
            (#{"notifications"} notifications)
        ])
    }
}
