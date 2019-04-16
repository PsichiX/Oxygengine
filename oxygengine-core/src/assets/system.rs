use crate::assets::database::AssetsDatabase;
use specs::{System, Write};

pub struct AssetsSystem;

impl<'s> System<'s> for AssetsSystem {
    type SystemData = Option<Write<'s, AssetsDatabase>>;

    fn run(&mut self, data: Self::SystemData) {
        if let Some(mut data) = data {
            data.process();
        }
    }
}
