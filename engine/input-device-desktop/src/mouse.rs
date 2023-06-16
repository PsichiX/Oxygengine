use backend::resource::DesktopAppEvents;
use core::{ecs::Universe, Scalar};
use glutin::event::{ElementState, MouseButton, WindowEvent};
use input::device::InputDevice;
use std::any::Any;

#[derive(Default)]
pub struct DesktopMouseInputDevice {
    position: (Scalar, Scalar),
    left_button: bool,
    right_button: bool,
    middle_button: bool,
}

impl InputDevice for DesktopMouseInputDevice {
    fn name(&self) -> &str {
        "mouse"
    }

    fn on_register(&mut self) {}

    fn on_unregister(&mut self) {}

    fn process(&mut self, universe: &mut Universe) {
        let events = universe.query_resources::<&DesktopAppEvents>();
        for event in events.iter() {
            match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    let state = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    match button {
                        MouseButton::Left => {
                            self.left_button = state;
                        }
                        MouseButton::Right => {
                            self.right_button = state;
                        }
                        MouseButton::Middle => {
                            self.middle_button = state;
                        }
                        _ => {}
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.position.0 = position.x as Scalar;
                    self.position.1 = position.y as Scalar;
                }
                _ => {}
            }
        }
    }

    fn query_axis(&self, name: &str) -> Option<Scalar> {
        match name {
            "x" => Some(self.position.0),
            "y" => Some(self.position.1),
            _ => None,
        }
    }

    fn query_trigger(&self, name: &str) -> Option<bool> {
        match name {
            "left" => Some(self.left_button),
            "right" => Some(self.right_button),
            "middle" => Some(self.middle_button),
            _ => None,
        }
    }

    fn query_text(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
