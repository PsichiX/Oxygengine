extern crate oxygengine_core as core;

pub mod device;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{device::*, resource::*, system::*};
}

use crate::{
    resource::InputController,
    system::{input_system, InputSystemResources},
};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
};

pub fn bundle_installer<PB, ICS>(
    builder: &mut AppBuilder<PB>,
    mut input_controller_setup: ICS,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    ICS: FnMut(&mut InputController),
{
    let mut input = InputController::default();
    input_controller_setup(&mut input);
    builder.install_resource(input);
    builder.install_system::<InputSystemResources>("input", input_system, &[])?;
    Ok(())
}
