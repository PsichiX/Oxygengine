extern crate oxygengine_core as core;

pub mod device;
pub mod resource;
pub mod system;

pub type Scalar = f32;

use crate::{resource::InputController, system::InputSystem};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b, ICS>(
    builder: &mut AppBuilder<'a, 'b>,
    mut input_controller_setup: ICS,
) where
    ICS: FnMut(&mut InputController),
{
    let mut input = InputController::default();
    input_controller_setup(&mut input);
    builder.install_resource(input);
    builder.install_system(InputSystem, "input", &[]);
}
