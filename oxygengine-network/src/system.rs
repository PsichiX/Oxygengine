use crate::{client::Client, resource::Network};
use core::ecs::{System, WriteExpect};
use std::marker::PhantomData;

pub struct NetworkSystem<C>
where
    C: Client + 'static,
{
    _phantom: PhantomData<C>,
}

impl<C> Default for NetworkSystem<C>
where
    C: Client + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<'s, C> System<'s> for NetworkSystem<C>
where
    C: Client + 'static,
{
    type SystemData = WriteExpect<'s, Network<C>>;

    fn run(&mut self, mut network: Self::SystemData) {
        network.process();
    }
}
