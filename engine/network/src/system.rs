use crate::{
    client::Client,
    resource::{Network, NetworkHost},
    server::Server,
};
use core::ecs::Universe;

pub type NetworkSystemResources<'a, C> = &'a mut Network<C>;

pub fn network_system<C>(universe: &mut Universe)
where
    C: Client + 'static,
{
    universe
        .query_resources::<NetworkSystemResources<C>>()
        .process();
}

pub type NetworkHostSystemResources<'a, S> = &'a mut NetworkHost<S>;

pub fn network_host_system<S>(universe: &mut Universe)
where
    S: Server + 'static,
{
    universe
        .query_resources::<NetworkHostSystemResources<S>>()
        .process();
}
