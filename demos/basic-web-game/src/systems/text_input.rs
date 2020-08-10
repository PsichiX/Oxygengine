use crate::resource::text_inputs::TextInputs;
use oxygengine::prelude::*;

pub struct TextInputWriterSystem;

impl<'s> System<'s> for TextInputWriterSystem {
    type SystemData = (Write<'s, TextInputs>, Read<'s, InputController>);

    fn run(&mut self, (mut text_inputs, controller): Self::SystemData) {
        if let Some(focused) = text_inputs.focused.clone() {
            if let Some(device) = controller.as_device::<WebKeyboardInputDevice>("keyboard") {
                let entry = text_inputs
                    .inputs
                    .entry(focused)
                    .or_insert(Default::default());
                entry.1 = entry.1.min(entry.0.len());
                for (key, code) in device.last_sequence() {
                    match code.as_str() {
                        "Backspace" => {
                            if !entry.0.is_empty() && entry.1 > 0 {
                                entry.1 -= 1;
                                entry.0.remove(entry.1);
                            }
                        }
                        "Delete" => {
                            if !entry.0.is_empty() && entry.1 < entry.0.len() {
                                entry.0.remove(entry.1);
                            }
                        }
                        "Enter" => {
                            entry.0.insert(entry.1, '\n');
                            entry.1 += 1;
                        }
                        "ArrowLeft" => {
                            entry.1 = if entry.1 > 0 { entry.1 - 1 } else { 0 };
                        }
                        "ArrowRight" => {
                            entry.1 = (entry.1 + 1).min(entry.0.len());
                        }
                        "Home" => {
                            entry.1 = 0;
                        }
                        "End" => {
                            entry.1 = entry.0.len();
                        }
                        _ => {
                            if !key.is_control() {
                                entry.0.insert(entry.1, *key);
                                entry.1 += 1;
                            }
                        }
                    }
                    entry.1 = entry.1.min(entry.0.len());
                }
            }
        }
    }
}

#[derive(Default)]
pub struct TextInputRendererSystem {
    phase: Scalar,
}

impl<'s> System<'s> for TextInputRendererSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Read<'s, TextInputs>,
        WriteStorage<'s, CompositeUiElement>,
    );

    fn run(&mut self, (lifecycle, text_inputs, mut ui_elements): Self::SystemData) {
        self.phase = (self.phase + lifecycle.delta_time_seconds()).fract();
        let cursor_shown = self.phase < 0.5;

        for ui_element in (&mut ui_elements).join() {
            for (id, (text, cursor_pos)) in &text_inputs.inputs {
                if let Some(ui_element) = ui_element.find_mut(id) {
                    if let UiElementType::Text(data) = &mut ui_element.element_type {
                        let is_focused = if let Some(focused) = &text_inputs.focused {
                            focused == id
                        } else {
                            false
                        };
                        let text = if cursor_shown && is_focused {
                            format!("{}|{}", &text[0..*cursor_pos], &text[*cursor_pos..])
                        } else {
                            text.to_owned()
                        };
                        data.text = text.into();
                    }
                }
            }
        }
    }
}
