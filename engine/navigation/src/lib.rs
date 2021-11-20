extern crate oxygengine_core as core;

pub mod component;
pub mod nav_mesh_asset_protocol;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{component::*, nav_mesh_asset_protocol::*, resource::*, system::*};
}

use crate::{
    component::{NavAgent, SimpleNavDriverTag},
    resource::NavMeshes,
    system::{
        nav_agent_maintain_system, simple_nav_driver_system, NavAgentMaintainSystemResources,
        SimpleNavDriverSystemResources,
    },
};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    ignite_proxy,
    prefab::PrefabManager,
};

ignite_proxy! {
    struct NavAgentId {}
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

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(NavMeshes::default());
    builder.install_system::<NavAgentMaintainSystemResources>(
        "nav-agent-maintain",
        nav_agent_maintain_system,
        &[],
    )?;
    builder.install_system::<SimpleNavDriverSystemResources>(
        "simple-nav-driver",
        simple_nav_driver_system,
        &["nav-agent-maintain"],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<SimpleNavDriverTag>("NavAgent");
    prefabs.register_component_factory::<NavAgent>("NavAgent");
}
