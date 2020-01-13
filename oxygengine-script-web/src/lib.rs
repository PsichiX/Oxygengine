extern crate oxygengine_core as core;
#[macro_use]
extern crate lazy_static;

mod component;
pub mod interface;
pub mod scriptable;
pub mod state;
pub mod web_api;

pub mod prelude {
    pub use crate::{interface::*, scriptable::*, state::*};
}
use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b, WSS>(builder: &mut AppBuilder<'a, 'b>, mut web_script_setup: WSS)
where
    WSS: FnMut(&mut WebScriptInterface),
{
    builder.install_component::<WebScriptComponent>();
    WebScriptInterface::with(|i| web_script_setup(i));
}
