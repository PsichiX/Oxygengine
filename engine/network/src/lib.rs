extern crate oxygengine_core as core;

pub mod client;
pub mod resource;
pub mod server;
pub mod system;

pub mod prelude {
    pub use crate::{client::*, resource::*, server::*, system::*};
}

use crate::{
    client::Client,
    resource::{Network, NetworkHost},
    server::Server,
    system::{
        network_host_system, network_system, NetworkHostSystemResources, NetworkSystemResources,
    },
};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
};

pub fn bundle_installer<PB, C, S>(
    builder: &mut AppBuilder<PB>,
    _: (),
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    C: Client + 'static,
    S: Server + 'static,
{
    builder.install_resource(Network::<C>::default());
    builder.install_resource(NetworkHost::<S>::default());
    builder.install_system::<NetworkSystemResources<C>>("network", network_system::<C>, &[])?;
    builder.install_system::<NetworkHostSystemResources<S>>(
        "network-host",
        network_host_system::<S>,
        &[],
    )?;
    Ok(())
}
