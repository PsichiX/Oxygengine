use crate::resource::InputController;
use core::ecs::Universe;

pub type InputSystemResources<'a> = &'a mut InputController;

pub fn input_system(universe: &mut Universe) {
    universe.query_resources::<&mut InputController>().process();
}
