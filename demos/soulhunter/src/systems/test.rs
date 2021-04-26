use crate::components::{animal_kind::AnimalKind, item_kind::ItemKind};
use oxygengine::prelude::*;

pub struct TestSystem;

impl<'s> System<'s> for TestSystem {
    type SystemData = (ReadStorage<'s, ItemKind>, ReadStorage<'s, AnimalKind>);

    fn run(&mut self, (_items, _animals): Self::SystemData) {}
}
