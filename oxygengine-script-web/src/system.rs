use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use core::{
    ecs::{world::EntitiesRes, LazyUpdate},
    prelude::*,
};
use std::marker::PhantomData;

pub struct WebScriptSystem<SD> {
    _phantom: PhantomData<SD>,
}

impl<'s, SD> System<'s> for WebScriptSystem<SD>
where
    SD: SystemData<'s>,
{
    type SystemData = (
        Read<'s, EntitiesRes>,
        Read<'s, LazyUpdate>,
        ReadStorage<'s, WebScriptComponent>,
        SD,
    );

    fn run(&mut self, (entities, lazy, components, data): Self::SystemData) {
        WebScriptInterface::run_systems(components, data);
        WebScriptInterface::maintain_entities(&entities, &lazy);
    }
}
