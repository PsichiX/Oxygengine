extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod component;
pub mod resource;
pub mod system;
pub mod ui_theme_asset_protocol;

// reexport macros.
pub use raui_core::{
    post_hooks, pre_hooks,
    prelude::{MessageData, Prefab, PropsData},
    unpack_named_slots, widget,
};

pub mod prelude {
    pub use crate::{
        component::*, resource::*, system::*, ui_theme_asset_protocol::*, UserInterfaceBundleSetup,
    };
}
pub mod raui {
    pub mod core {
        pub use raui_core::*;
    }
    pub mod material {
        pub use raui_material::*;
    }
}

use crate::{
    component::UserInterfaceView,
    resource::UserInterface,
    system::{user_interface_system, UserInterfaceSystemCache, UserInterfaceSystemResources},
};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Universe,
    },
    prefab::PrefabManager,
};
use raui_core::application::ProcessContext;

#[derive(Default)]
pub struct UserInterfaceBundleSetup {
    user_interface: UserInterface,
    process_context_setup:
        Option<Box<dyn Fn(&Universe, &mut ProcessContext) + Send + Sync + 'static>>,
}

impl UserInterfaceBundleSetup {
    pub fn user_interface(mut self, v: UserInterface) -> Self {
        self.user_interface = v;
        self
    }

    pub fn process_context_setup<F>(mut self, f: F) -> Self
    where
        F: Fn(&Universe, &mut ProcessContext) + Send + Sync + 'static,
    {
        self.process_context_setup = Some(Box::new(f));
        self
    }
}

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    setup: UserInterfaceBundleSetup,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    let UserInterfaceBundleSetup {
        user_interface,
        process_context_setup,
    } = setup;
    let mut cache = UserInterfaceSystemCache::default();
    cache.process_context_setup = process_context_setup;
    builder.install_resource(user_interface);
    builder.install_resource(cache);
    builder.install_system::<UserInterfaceSystemResources>(
        "user-interface",
        user_interface_system,
        &[],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<UserInterfaceView>("UserInterfaceView");
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(ui_theme_asset_protocol::UiThemeAssetProtocol);
}
