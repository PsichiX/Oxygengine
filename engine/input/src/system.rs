use crate::{
    component::InputStackInstance,
    resources::{controller::InputController, stack::InputStack},
};
use core::ecs::{life_cycle::EntityChanges, Comp, Universe, WorldRef};

pub type InputSystemResources<'a> = (
    WorldRef,
    &'a mut InputController,
    &'a mut InputStack,
    &'a EntityChanges,
    Comp<&'a mut InputStackInstance>,
);

pub fn input_system(universe: &mut Universe) {
    let (world, mut controller, mut stack, entity_changes, ..) =
        universe.query_resources::<InputSystemResources>();

    for (entity, instance) in world.query::<&mut InputStackInstance>().iter() {
        if let InputStackInstance::Setup(listener) = &instance {
            let mut listener = listener.to_owned();
            listener.bound_entity = Some(entity);
            *instance = InputStackInstance::Listener(stack.register(listener));
        }
    }

    controller.process(universe);
    stack.process(&controller, &entity_changes);
}
