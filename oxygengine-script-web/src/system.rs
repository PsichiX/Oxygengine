use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use core::{
    ecs::{world::EntitiesRes, LazyUpdate},
    prelude::*,
};

pub struct WebScriptSystem;

impl<'s> System<'s> for WebScriptSystem {
    type SystemData = (
        Read<'s, EntitiesRes>,
        Read<'s, LazyUpdate>,
        ReadStorage<'s, WebScriptComponent>,
    );

    fn run(&mut self, (entities, lazy, components): Self::SystemData) {
        for (entity, component) in (&entities, &components).join() {}

        WebScriptInterface::build_entities(&entities, &lazy);
    }
}
