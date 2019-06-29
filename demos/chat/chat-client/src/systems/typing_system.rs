use crate::{components::Typing, resources::text_input_res::TextInputRes};
use oxygengine::prelude::*;

pub struct TypingSystem;

impl<'s> System<'s> for TypingSystem {
    type SystemData = (
        Write<'s, TextInputRes>,
        ReadStorage<'s, Typing>,
        WriteStorage<'s, CompositeRenderable>,
    );

    fn run(&mut self, (mut input_res, typing, mut renderable): Self::SystemData) {
        if let Some(typing_value) = input_res.read_typing() {
            for (_, renderable) in (&typing, &mut renderable).join() {
                if let Renderable::Text(text) = &mut renderable.0 {
                    text.text = format!("> {}", typing_value).into();
                }
            }
        }
    }
}
