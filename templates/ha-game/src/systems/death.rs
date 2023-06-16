use crate::components::health::*;
use oxygengine::prelude::*;

pub type DeathSystemResources<'a> = (WorldRef, &'a mut UniverseCommands, Comp<&'a Health>);

// TODO: rename to Grim Reaper to not keep sad system names in game template.
pub fn death_system(universe: &mut Universe) {
    let (world, mut commands, ..) = universe.query_resources::<DeathSystemResources>();

    for (entity, health) in world.query::<&Health>().iter() {
        if health.0 == 0 {
            commands.schedule(DespawnEntity(entity));
        }
    }
}
