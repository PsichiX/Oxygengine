extern crate oxygengine_core as core;

pub mod component;
pub mod nav_mesh_asset_protocol;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{component::*, nav_mesh_asset_protocol::*, resource::*, system::*};
}

pub type Scalar = f64;

use crate::{
    resource::NavMeshesRes,
    system::{NavAgentMaintainSystem, SimpleNavDriverSystem},
};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>) {
    builder.install_resource(NavMeshesRes::default());
    builder.install_system(NavAgentMaintainSystem, "nav-agent-maintain", &[]);
    builder.install_system(
        SimpleNavDriverSystem,
        "simple-nav-driver",
        &["nav-agent-maintain"],
    );
}
