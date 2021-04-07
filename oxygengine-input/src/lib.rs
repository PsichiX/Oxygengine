extern crate oxygengine_core as core;

pub mod device;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{device::*, resource::*, system::*};
}

use crate::{resource::InputController, system::InputSystem};
use core::app::AppBuilder;

pub fn bundle_installer<ICS>(builder: &mut AppBuilder, mut input_controller_setup: ICS)
where
    ICS: FnMut(&mut InputController),
{
    let mut input = InputController::default();
    input_controller_setup(&mut input);
    builder.install_resource(input);
    builder.install_system(InputSystem, "input", &[]);
}
