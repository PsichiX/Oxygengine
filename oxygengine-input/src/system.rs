use crate::resource::InputController;
use core::ecs::{System, Write};

pub struct InputSystem;

impl<'s> System<'s> for InputSystem {
    type SystemData = Write<'s, InputController>;

    fn run(&mut self, mut input: Self::SystemData) {
        (&mut input).process();
    }
}
