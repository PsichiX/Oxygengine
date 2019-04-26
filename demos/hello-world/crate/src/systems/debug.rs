use crate::macros::*;
use oxygengine::prelude::*;

pub struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = Read<'s, InputController>;

    fn run(&mut self, input: Self::SystemData) {
        console_log!("mx: {:?}", input.axis("mouse-x"));
        console_log!("my: {:?}", input.axis("mouse-y"));
    }
}
