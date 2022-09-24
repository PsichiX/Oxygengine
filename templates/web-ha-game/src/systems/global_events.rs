use crate::{components::player::*, resources::*};
use oxygengine::prelude::*;

pub type GlobalEventsSystemResources<'a> =
    (WorldRef, &'a mut Events<GlobalEvent>, Comp<&'a mut Player>);

pub fn global_events_system(universe: &mut Universe) {
    let (world, mut events, ..) = universe.query_resources::<GlobalEventsSystemResources>();

    for message in events.consume() {
        match message {
            GlobalEvent::LevelUp(entity, levels) => {
                if levels > 0 {
                    if let Ok(mut player) = world.get::<&mut Player>(entity) {
                        player.level += levels;
                    }
                }
            }
        }
    }
}
