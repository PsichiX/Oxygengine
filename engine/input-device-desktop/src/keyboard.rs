use backend::resource::DesktopAppEvents;
use core::{ecs::Universe, Scalar};
use glutin::event::{ElementState, WindowEvent};
use input::device::InputDevice;
use std::{any::Any, collections::HashSet};

#[derive(Default)]
pub struct DesktopKeyboardInputDevice {
    keys: HashSet<String>,
    text: String,
}

impl InputDevice for DesktopKeyboardInputDevice {
    fn name(&self) -> &str {
        "keyboard"
    }

    fn on_register(&mut self) {}

    fn on_unregister(&mut self) {}

    fn process(&mut self, universe: &mut Universe) {
        let events = universe.query_resources::<&DesktopAppEvents>();
        for event in events.iter() {
            if let WindowEvent::KeyboardInput { input, .. } = event {
                let key = input
                    .virtual_keycode
                    .as_ref()
                    .map(|keycode| format!("{:?}", keycode))
                    .unwrap_or_else(|| format!("#{}", input.scancode));
                match input.state {
                    ElementState::Pressed => {
                        self.keys.insert(key);
                    }
                    ElementState::Released => {
                        self.keys.remove(&key);
                    }
                }
            }
        }
        self.text.clear();
    }

    fn query_axis(&self, name: &str) -> Option<Scalar> {
        Some(if self.keys.contains(name) { 1.0 } else { 0.0 })
    }

    fn query_trigger(&self, name: &str) -> Option<bool> {
        Some(self.keys.contains(name))
    }

    fn query_text(&self) -> Option<String> {
        Some(self.text.to_owned())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
