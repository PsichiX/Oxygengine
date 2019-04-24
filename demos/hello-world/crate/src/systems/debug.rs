use crate::macros::*;
use oxygengine::{composite_renderer::composite_renderer::*, prelude::*};

pub struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = ReadExpect<'s, PlatformCompositeRenderer>;

    fn run(&mut self, renderer: Self::SystemData) {
        console_log!("{:#?}", renderer.state().stats());
    }
}
