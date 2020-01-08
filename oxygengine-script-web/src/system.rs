use crate::component::WebScriptComponent;
use core::{ecs::world::EntitiesRes, prelude::*};

pub struct WebScriptSystem;

impl<'s> System<'s> for WebScriptSystem {
    type SystemData = (Write<'s, EntitiesRes>, ReadStorage<'s, WebScriptComponent>);

    fn run(&mut self, (entities, components): Self::SystemData) {}
}
