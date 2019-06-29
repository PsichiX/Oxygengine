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
    system::{NetworkHostSystem, NetworkSystem},
};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b, C, S>(builder: &mut AppBuilder<'a, 'b>, _: ())
where
    C: Client + 'static,
    S: Server + 'static,
{
    builder.install_resource(Network::<C>::default());
    builder.install_resource(NetworkHost::<S>::default());
    builder.install_system(NetworkSystem::<C>::default(), "network", &[]);
    builder.install_system(NetworkHostSystem::<S>::default(), "network_host", &[]);
}
