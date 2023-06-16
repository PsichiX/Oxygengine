use crate::device::InputDevice;
use core::{ecs::Universe, Scalar};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputMappings {
    /// {device name: {name from: name to}}
    #[serde(default)]
    pub axes: HashMap<String, HashMap<String, String>>,
    /// {device name: {name from: name to}}
    #[serde(default)]
    pub triggers: HashMap<String, HashMap<String, String>>,
}

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

    #[inline]
    pub fn press(self) -> Self {
        match self {
            Self::Idle => Self::Pressed,
            Self::Pressed => Self::Hold,
            Self::Hold => Self::Hold,
            Self::Released => Self::Pressed,
        }
    }

    #[inline]
    pub fn release(self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Pressed => Self::Released,
            Self::Hold => Self::Released,
            Self::Released => Self::Idle,
        }
    }

    #[inline]
    pub fn progress(self, press: bool) -> Self {
        if press {
            self.press()
        } else {
            self.release()
        }
    }

    #[inline]
    pub fn priority(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Pressed => 1,
            Self::Hold => 2,
            Self::Released => 1,
        }
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
        self.devices.get(id).map(|device| device.as_ref())
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

    pub fn map_config(&mut self, config: InputMappings) {
        for (device, mappings) in config.axes {
            for (name_from, name_to) in mappings {
                self.map_axis(&name_from, &device, &name_to);
            }
        }
        for (device, mappings) in config.triggers {
            for (name_from, name_to) in mappings {
                self.map_trigger(&name_from, &device, &name_to);
            }
        }
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

    pub fn multi_axis<const N: usize>(&self, names: [&str; N]) -> [Option<Scalar>; N] {
        let mut result = [None; N];
        for i in 0..N {
            result[i] = self.axis(names[i]);
        }
        result
    }

    pub fn mirror_multi_axis<const N: usize>(
        &self,
        names: [(&str, &str); N],
    ) -> [Option<Scalar>; N] {
        let mut result = [None; N];
        for i in 0..N {
            let name = names[i];
            result[i] = match (self.axis(name.0), self.axis(name.1)) {
                (None, None) => None,
                (Some(v), None) => Some(-v),
                (None, Some(v)) => Some(v),
                (Some(a), Some(b)) => Some(b - a),
            };
        }
        result
    }

    pub fn axis_or_default(&self, name: &str) -> Scalar {
        self.axis(name).unwrap_or(0.0)
    }

    pub fn multi_axis_or_default<const N: usize>(&self, names: [&str; N]) -> [Scalar; N] {
        let mut result = [Default::default(); N];
        for i in 0..N {
            result[i] = self.axis_or_default(names[i]);
        }
        result
    }

    pub fn mirror_multi_axis_or_default<const N: usize>(
        &self,
        names: [(&str, &str); N],
    ) -> [Scalar; N] {
        let mut result = [Default::default(); N];
        for i in 0..N {
            let name = names[i];
            result[i] = self.axis_or_default(name.0) - self.axis_or_default(name.1);
        }
        result
    }

    pub fn set_axis(&mut self, name: &str, value: Scalar) {
        self.axes.insert(name.to_owned(), value);
    }

    pub fn trigger(&self, name: &str) -> Option<TriggerState> {
        self.triggers.get(name).cloned()
    }

    pub fn multi_trigger<const N: usize>(&self, names: [&str; N]) -> [Option<TriggerState>; N] {
        let mut result = [None; N];
        for i in 0..N {
            result[i] = self.trigger(names[i]);
        }
        result
    }

    pub fn priority_trigger<const N: usize>(&self, names: [&str; N]) -> Option<TriggerState> {
        let mut result = None;
        for name in names.iter() {
            result = match (result, self.trigger(name)) {
                (None, v) => v,
                (Some(a), Some(b)) => {
                    if b.priority() > a.priority() {
                        Some(b)
                    } else {
                        Some(a)
                    }
                }
                _ => result,
            };
        }
        result
    }

    pub fn trigger_or_default(&self, name: &str) -> TriggerState {
        self.trigger(name).unwrap_or(TriggerState::Idle)
    }

    pub fn multi_trigger_or_default<const N: usize>(&self, names: [&str; N]) -> [TriggerState; N] {
        let mut result = [Default::default(); N];
        for i in 0..N {
            result[i] = self.trigger_or_default(names[i]);
        }
        result
    }

    pub fn priority_trigger_or_default<const N: usize>(&self, names: [&str; N]) -> TriggerState {
        let mut result = TriggerState::Idle;
        for name in names.iter() {
            let state = self.trigger_or_default(name);
            if state.priority() > result.priority() {
                result = state;
            }
        }
        result
    }

    pub fn set_trigger(&mut self, name: &str, value: TriggerState) {
        self.triggers.insert(name.to_owned(), value);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn process(&mut self, universe: &mut Universe) {
        for device in self.devices.values_mut() {
            device.process(universe);
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
                    let next = prev.progress(value);
                    self.triggers.insert(name_from.to_owned(), next);
                }
            }
        }
    }
}
