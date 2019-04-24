use crate::{device::InputDevice, Scalar};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub struct InputController {
    devices: HashMap<TypeId, Box<InputDevice>>,
    axes: HashMap<String, Scalar>,
    triggers: HashMap<String, bool>,
}

impl InputController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<D>(&mut self, mut device: D)
    where
        D: InputDevice + 'static,
    {
        device.on_register();
        let tid = device.type_id();
        self.devices.insert(tid, Box::new(device));
    }

    pub fn unregister<D>(&mut self) -> Option<Box<dyn InputDevice>>
    where
        D: InputDevice + 'static,
    {
        if let Some(mut device) = self.devices.remove(&TypeId::of::<D>()) {
            device.on_unregister();
            Some(device)
        } else {
            None
        }
    }

    pub fn axis(&self, name: &str) -> Option<Scalar> {
        self.axes.get(name).map(|v| *v)
    }

    pub fn axis_or_default(&self, name: &str) -> Scalar {
        self.axis(name).unwrap_or(0.0)
    }

    pub fn set_axis(&mut self, name: &str, value: Scalar) {
        self.axes.insert(name.to_owned(), value);
    }

    pub fn trigger(&self, name: &str) -> Option<bool> {
        self.triggers.get(name).map(|v| *v)
    }

    pub fn trigger_or_default(&self, name: &str) -> bool {
        self.trigger(name).unwrap_or(false)
    }

    pub fn set_trigger(&mut self, name: &str, value: bool) {
        self.triggers.insert(name.to_owned(), value);
    }

    pub fn process(&mut self) {
        for device in self.devices.values_mut() {
            device.process();
        }
        self.axes.clear();
        self.triggers.clear();
        for device in self.devices.values() {
            for (name, value) in device.query_axes() {
                self.axes.insert(name.to_string(), *value);
            }
            for (name, value) in device.query_triggers() {
                self.triggers.insert(name.to_string(), *value);
            }
        }
    }
}
