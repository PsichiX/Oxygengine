pub mod components;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::{components::*, resources::*, systems::*, AiSystemInstallerSetup};
}

pub use emergent;

use crate::{
    components::AiInstance,
    resources::AiBehaviors,
    systems::{ai_system, AiSystemResources},
};
use oxygengine_core::{
    app::AppBuilder,
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Component,
    },
    prefab::PrefabManager,
};

pub struct AiSystemInstallerSetup<'a, C>
where
    C: Component,
{
    pub postfix: &'a str,
    pub behaviors: AiBehaviors<C>,
}

pub fn ai_system_installer<PB, C>(
    builder: &mut AppBuilder<PB>,
    setup: AiSystemInstallerSetup<C>,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    C: Component,
{
    builder.install_resource(setup.behaviors);
    builder.install_system::<AiSystemResources<C>>(
        &format!("ai-system-{}", setup.postfix),
        ai_system::<C>,
        &[],
    )?;
    Ok(())
}

pub fn prefabs_installer<C>(postfix: &str, prefabs: &mut PrefabManager)
where
    C: Component + Default,
{
    prefabs.register_component_factory::<AiInstance<C>>(&format!("AiInstance-{}", postfix));
}
