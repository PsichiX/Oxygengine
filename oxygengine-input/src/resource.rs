use crate::device::InputDevice;
use core::Scalar;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TriggerState {
    Idle,
    Pressed,
    Hold,
    Released,
}

impl Default for TriggerState {
    fn default() -> Self {
        Self::Idle
    }
}

impl TriggerState {
    #[inline]
    pub fn is_on(self) -> bool {
        self == TriggerState::Pressed || self == TriggerState::Hold
    }

    #[inline]
    pub fn is_off(self) -> bool {
        !self.is_on()
    }

    #[inline]
    pub fn is_idle(self) -> bool {
        self == TriggerState::Idle
    }

    #[inline]
    pub fn is_pressed(self) -> bool {
        self == TriggerState::Pressed
    }

    #[inline]
    pub fn is_hold(self) -> bool {
        self == TriggerState::Hold
    }

    #[inline]
    pub fn is_released(self) -> bool {
        self == TriggerState::Released
    }
}

#[derive(Default)]
pub struct InputController {
    devices: HashMap<String, Box<dyn InputDevice>>,
    mapping_axes: HashMap<String, (String, String)>,
    mapping_triggers: HashMap<String, (String, String)>,
    axes: HashMap<String, Scalar>,
    triggers: HashMap<String, TriggerState>,
    text: String,
}

impl InputController {
    pub fn register<D>(&mut self, mut device: D)
    where
        D: InputDevice + 'static,
    {
        device.on_register();
        self.devices
            .insert(device.name().to_owned(), Box::new(device));
    }

    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn InputDevice>> {
        if let Some(mut device) = self.devices.remove(name) {
            device.on_unregister();
            Some(device)
        } else {
            None
        }
    }

    pub fn device(&self, id: &str) -> Option<&dyn InputDevice> {
        if let Some(device) = self.devices.get(id) {
            Some(device.borrow())
        } else {
            None
        }
    }

    pub fn as_device<T>(&self, id: &str) -> Option<&T>
    where
        T: InputDevice,
    {
        if let Some(device) = self.devices.get(id) {
            device.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn mapping_axes(&self) -> impl Iterator<Item = (&str, (&str, &str))> {
        self.mapping_axes
            .iter()
            .map(|(k, (a, b))| (k.as_str(), (a.as_str(), b.as_str())))
    }

    pub fn mapping_triggers(&self) -> impl Iterator<Item = (&str, (&str, &str))> {
        self.mapping_triggers
            .iter()
            .map(|(k, (a, b))| (k.as_str(), (a.as_str(), b.as_str())))
    }

    pub fn map_axis(&mut self, name_from: &str, device: &str, name_to: &str) {
        self.mapping_axes.insert(
            name_from.to_owned(),
            (device.to_owned(), name_to.to_owned()),
        );
    }

    pub fn unmap_axis(&mut self, name: &str) {
        self.mapping_axes.remove(name);
    }

    pub fn map_trigger(&mut self, name_from: &str, device: &str, name_to: &str) {
        self.mapping_triggers.insert(
            name_from.to_owned(),
            (device.to_owned(), name_to.to_owned()),
        );
    }

    pub fn unmap_trigger(&mut self, name: &str) {
        self.mapping_triggers.remove(name);
    }

    pub fn axes(&self) -> impl Iterator<Item = (&str, Scalar)> {
        self.axes.iter().map(|(k, v)| (k.as_str(), *v))
    }

    pub fn triggers(&self) -> impl Iterator<Item = (&str, TriggerState)> {
        self.triggers.iter().map(|(k, v)| (k.as_str(), *v))
    }

    pub fn axis(&self, name: &str) -> Option<Scalar> {
        self.axes.get(name).cloned()
    }

    pub fn axis_or_default(&self, name: &str) -> Scalar {
        self.axis(name).unwrap_or(0.0)
    }

    pub fn set_axis(&mut self, name: &str, value: Scalar) {
        self.axes.insert(name.to_owned(), value);
    }

    pub fn trigger(&self, name: &str) -> Option<TriggerState> {
        self.triggers.get(name).cloned()
    }

    pub fn trigger_or_default(&self, name: &str) -> TriggerState {
        self.trigger(name).unwrap_or(TriggerState::Idle)
    }

    pub fn set_trigger(&mut self, name: &str, value: TriggerState) {
        self.triggers.insert(name.to_owned(), value);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn process(&mut self) {
        for device in self.devices.values_mut() {
            device.process();
        }
        self.text.clear();
        for device in self.devices.values() {
            if let Some(text) = device.query_text() {
                self.text.push_str(&text);
            }
        }
        self.axes.clear();
        for (name_from, (dev, name_to)) in &self.mapping_axes {
            if let Some(device) = self.devices.get(dev) {
                if let Some(value) = device.query_axis(name_to) {
                    self.axes.insert(name_from.to_owned(), value);
                }
            }
        }
        for (name_from, (dev, name_to)) in &self.mapping_triggers {
            if let Some(device) = self.devices.get(dev) {
                if let Some(value) = device.query_trigger(name_to) {
                    let prev = self.triggers.get(name_from).unwrap_or(&TriggerState::Idle);
                    match (prev, value) {
                        (TriggerState::Idle, true) | (TriggerState::Released, true) => {
                            self.triggers
                                .insert(name_from.to_owned(), TriggerState::Pressed);
                        }
                        (TriggerState::Pressed, true) | (TriggerState::Pressed, false) => {
                            self.triggers
                                .insert(name_from.to_owned(), TriggerState::Hold);
                        }
                        (TriggerState::Hold, false) => {
                            self.triggers
                                .insert(name_from.to_owned(), TriggerState::Released);
                        }
                        (TriggerState::Released, false) => {
                            self.triggers
                                .insert(name_from.to_owned(), TriggerState::Idle);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
