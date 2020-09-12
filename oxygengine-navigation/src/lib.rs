extern crate oxygengine_core as core;

pub mod component;
pub mod nav_mesh_asset_protocol;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{component::*, nav_mesh_asset_protocol::*, resource::*, system::*};
}

use crate::{
    component::NavAgent,
    resource::NavMeshesRes,
    system::{NavAgentMaintainSystem, SimpleNavDriverSystem},
};
use core::{app::AppBuilder, ignite_proxy, prefab::PrefabManager};

ignite_proxy! {
    struct NavAgentID {}
}

ignite_proxy! {
    struct NavVec3 {
        pub x: Scalar,
        pub y: Scalar,
        pub z: Scalar,
    }
}

ignite_proxy! {
    struct NavTriangle {
        pub first: u32,
        pub second: u32,
        pub third: u32,
    }
}

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>) {
    builder.install_resource(NavMeshesRes::default());
    builder.install_system(NavAgentMaintainSystem::default(), "nav-agent-maintain", &[]);
    builder.install_system(
        SimpleNavDriverSystem,
        "simple-nav-driver",
        &["nav-agent-maintain"],
    );
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<NavAgent>("NavAgent");
}
