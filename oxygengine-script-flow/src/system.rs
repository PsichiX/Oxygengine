use crate::resource::FlowManager;
use core::ecs::{System, Write};

#[derive(Default)]
pub struct FlowSystem;

impl<'s> System<'s> for FlowSystem {
    type SystemData = Write<'s, FlowManager>;

    fn run(&mut self, mut manager: Self::SystemData) {
        drop(manager.process_events());
    }
}
