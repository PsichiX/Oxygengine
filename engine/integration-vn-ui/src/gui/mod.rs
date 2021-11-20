pub mod backgrounds;
pub mod characters;
pub mod dialogue;
pub mod game_menu;
pub mod main_menu;
pub mod overlay;

use crate::{
    gui::{
        backgrounds::visual_novel_backgrounds_container,
        characters::visual_novel_characters_container, dialogue::visual_novel_dialogue_container,
        overlay::visual_novel_overlay_container,
    },
    VisualNovelGuiProps,
};
use oxygengine_user_interface::raui::core::prelude::*;

macro_rules! impl_child_widget {
    ($name:ident, $component:ident, $props:expr) => {
        let $name = if $props.$name {
            if $name.is_none() {
                widget! { (#{stringify!($name)} $component) }
            } else {
                $name
            }
        } else {
            Default::default()
        };
    };
}

pub fn visual_novel_gui(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {
        overlay,
        // main_menu,
        // game_menu,
        dialogue,
        characters,
        backgrounds
    });

    let gui_props = props.read_cloned_or_default::<VisualNovelGuiProps>();

    impl_child_widget!(overlay, visual_novel_overlay_container, gui_props);
    // impl_child_widget!(main_menu, visual_novel_main_menu, gui_props);
    // impl_child_widget!(game_menu, visual_novel_game_menu, gui_props);
    impl_child_widget!(dialogue, visual_novel_dialogue_container, gui_props);
    impl_child_widget!(characters, visual_novel_characters_container, gui_props);
    impl_child_widget!(backgrounds, visual_novel_backgrounds_container, gui_props);

    widget! {
        (#{key} content_box [
            {backgrounds}
            {characters}
            {dialogue}
            // {game_menu}
            // {main_menu}
            {overlay}
        ])
    }
}
