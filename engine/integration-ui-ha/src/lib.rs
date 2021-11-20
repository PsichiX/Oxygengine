pub mod components;
pub mod raui_renderer;
pub mod systems;

pub mod prelude {
    pub use crate::{
        components::*,
        systems::{render_ui_stage::*, user_interface_sync::*},
    };
}

use crate::{
    components::HaUserInterfaceSync,
    systems::{
        render_ui_stage::{
            ha_render_ui_stage_system, HaRenderUiStageSystemCache, HaRenderUiStageSystemResources,
        },
        user_interface_sync::{ha_user_interface_sync_system, HaUserInterfaceSyncSystemResources},
    },
};
use oxygengine_core::prelude::*;

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(HaRenderUiStageSystemCache::default());
    builder.install_system::<HaUserInterfaceSyncSystemResources>(
        "user-interface-sync",
        ha_user_interface_sync_system,
        &["renderer"],
    )?;
    builder.install_system::<HaRenderUiStageSystemResources>(
        "render-ui-stage",
        ha_render_ui_stage_system,
        &["renderer"],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<HaUserInterfaceSync>("HaUserInterfaceSync");
}
