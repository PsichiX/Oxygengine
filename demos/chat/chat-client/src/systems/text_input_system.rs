use crate::resources::text_input_res::TextInputRes;
use oxygengine::prelude::*;

pub struct TextInputSystem;

impl<'s> System<'s> for TextInputSystem {
    type SystemData = (Write<'s, TextInputRes>, Read<'s, InputController>);

    fn run(&mut self, (mut input_res, input): Self::SystemData) {
        if let Some(keyboard) = input.as_device::<WebKeyboardInputDevice>("keyboard") {
            for (key, code) in keyboard.last_sequence() {
                if key.is_alphanumeric() || key.is_whitespace() {
                    input_res.push_typing(*key);
                    info!("* TEXT: {:?}", input_res.typing());
                } else if code == "Backspace" {
                    input_res.pop_typing();
                    info!("* TEXT: {:?}", input_res.typing());
                } else if code == "Enter" {
                    input_res.store_typing();
                }
            }
        }
    }
}
