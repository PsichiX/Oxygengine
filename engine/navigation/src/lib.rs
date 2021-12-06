extern crate oxygengine_core as core;

pub mod asset_protocols;
pub mod components;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::{
        asset_protocols::{nav_grid::*, nav_mesh::*, *},
        components::*,
        resources::{nav_grids::*, nav_meshes::*, *},
        systems::*,
    };
}

use crate::{
    asset_protocols::{nav_grid::NavGridAssetProtocol, nav_mesh::NavMeshAssetProtocol},
    components::{NavAgent, SimpleNavDriverTag},
    resources::{nav_grids::NavGrids, nav_meshes::NavMeshes},
    systems::{
        nav_agent_maintain_system, simple_nav_driver_system, NavAgentMaintainSystemResources,
        SimpleNavDriverSystemResources,
    },
};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
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
    builder.install_resource(NavGrids::default());
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

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(NavMeshAssetProtocol);
    database.register(NavGridAssetProtocol);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<SimpleNavDriverTag>("NavAgent");
    prefabs.register_component_factory::<NavAgent>("NavAgent");
}
