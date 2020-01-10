#[macro_use]
extern crate oxygengine_core as core;
#[macro_use]
extern crate lazy_static;

pub mod component;
pub mod interface;
pub mod state;
pub mod web_api;

pub mod prelude {
    pub use crate::{component::*, interface::*, state::*};
}
use core::app::AppBuilder;
use crate::component::WebScriptComponent;

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>, _: ()) {
    builder.install_component::<WebScriptComponent>();
}
