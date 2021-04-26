use oxygengine::{prelude::*, user_interface::raui::core::widget::utils::Color as RauiColor};

pub fn rgba_to_raui_color(r: u8, g: u8, b: u8, a: u8) -> RauiColor {
    RauiColor {
        r: r as Scalar / 255.0,
        g: g as Scalar / 255.0,
        b: b as Scalar / 255.0,
        a: a as Scalar / 255.0,
    }
}
