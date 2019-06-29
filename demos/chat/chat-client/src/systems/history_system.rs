use crate::{components::History, resources::history_res::HistoryRes};
use oxygengine::prelude::*;

pub struct HistorySystem;

impl<'s> System<'s> for HistorySystem {
    type SystemData = (
        Write<'s, HistoryRes>,
        ReadStorage<'s, History>,
        WriteStorage<'s, CompositeRenderable>,
    );

    fn run(&mut self, (mut history_res, history, mut renderable): Self::SystemData) {
        if let Some(history_value) = history_res.read_text() {
            for (_, renderable) in (&history, &mut renderable).join() {
                if let Renderable::Text(text) = &mut renderable.0 {
                    text.text = history_value.to_owned().into();
                }
            }
        }
    }
}
