use crate::{assets::database::AssetsDatabase, ecs::Universe};

pub type AssetsSystemResources<'a> = &'a mut AssetsDatabase;

pub fn assets_system(universe: &mut Universe) {
    if let Some(mut database) = universe.resource_mut::<AssetsDatabase>() {
        database.process();
    }
}
