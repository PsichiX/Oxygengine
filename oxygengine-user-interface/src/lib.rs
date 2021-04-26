extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod component;
pub mod resource;
pub mod system;

// reexport macros.
pub use raui_core::{destruct, post_hooks, pre_hooks, unpack_named_slots, widget, widget_wrap};

pub mod prelude {
    pub use crate::{component::*, resource::*, system::*};
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
    component::UserInterfaceView, resource::UserInterfaceRes, system::UserInterfaceSystem,
};
use core::{app::AppBuilder, prefab::PrefabManager};

pub fn bundle_installer(builder: &mut AppBuilder, resource: UserInterfaceRes) {
    builder.install_resource(resource);
    builder.install_system(UserInterfaceSystem::default(), "user-interface", &[]);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<UserInterfaceView>("UserInterfaceView");
}
