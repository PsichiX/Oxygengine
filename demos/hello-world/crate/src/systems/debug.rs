use crate::macros::*;
use oxygengine::prelude::*;

pub struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = ReadExpect<'s, WebCompositeRenderer>;

    fn run(&mut self, renderer: Self::SystemData) {
        console_log!("{:#?}", renderer.state().stats());
    }
}
