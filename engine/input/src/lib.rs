extern crate oxygengine_core as core;

pub mod component;
pub mod device;
pub mod resources;
pub mod system;

pub mod prelude {
    pub use crate::{
        component::*,
        device::*,
        resources::{controller::*, stack::*},
        system::*,
    };
}

use crate::{
    component::InputStackInstance,
    resources::{controller::InputController, stack::InputStack},
    system::{input_system, InputSystemResources},
};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    prefab::PrefabManager,
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
    builder.install_resource(InputStack::default());
    builder.install_system::<InputSystemResources>("input", input_system, &[])?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<InputStackInstance>("InputStackInstance");
}
