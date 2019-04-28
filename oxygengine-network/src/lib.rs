extern crate oxygengine_core as core;

pub mod client;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{client::*, resource::*, system::*};
}

use crate::{client::Client, resource::Network, system::NetworkSystem};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b, C>(builder: &mut AppBuilder<'a, 'b>, version: u32)
where
    C: Client + 'static,
{
    builder.install_resource(Network::<C>::new(version));
    builder.install_system(NetworkSystem::<C>::default(), "network", &[]);
}
