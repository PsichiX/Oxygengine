use crate::{component::InputStackInstance, resources::controller::*};
use core::{
    ecs::{life_cycle::EntityChanges, Entity},
    id::ID,
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type InputStackListenerId = ID<InputStackListener>;
pub type InputStackMappings = HashSet<String>;
pub type InputStackScaledMappings = HashMap<String, Scalar>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStackTrigger {
    #[serde(default)]
    pub consume: bool,
    #[serde(default)]
    mappings: InputStackMappings,
    #[serde(skip)]
    state: TriggerState,
}

impl InputStackTrigger {
    pub fn new(mappings: impl Into<InputStackMappings>) -> Self {
        Self {
            consume: false,
            mappings: mappings.into(),
            state: Default::default(),
        }
    }

    pub fn mappings(&self) -> &InputStackMappings {
        &self.mappings
    }

    pub fn state(&self) -> TriggerState {
        self.state
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStackChannel {
    #[serde(default)]
    mappings: InputStackScaledMappings,
    #[serde(skip)]
    state: Scalar,
}

impl InputStackChannel {
    pub fn new(mappings: impl Into<InputStackScaledMappings>) -> Self {
        Self {
            mappings: mappings.into(),
            state: 0.0,
        }
    }

    pub fn mappings(&self) -> &InputStackScaledMappings {
        &self.mappings
    }

    pub fn state(&self) -> Scalar {
        self.state
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStackAxes {
    #[serde(default)]
    pub consume: bool,
    #[serde(default)]
    channels: Vec<InputStackChannel>,
}

impl InputStackAxes {
    pub fn new(channels: impl Iterator<Item = impl Into<InputStackChannel>>) -> Self {
        Self {
            consume: false,
            channels: channels.map(|item| item.into()).collect::<Vec<_>>(),
        }
    }

    pub fn new_single(channel: impl Into<InputStackChannel>) -> Self {
        Self {
            consume: false,
            channels: vec![channel.into()],
        }
    }

    pub fn channels(&self) -> &[InputStackChannel] {
        &self.channels
    }

    pub fn channels_state_or_default<const N: usize>(&self) -> [Scalar; N] {
        let mut result = [0.0; N];
        for (channel, result) in self.channels.iter().zip(result.iter_mut()) {
            *result = channel.state;
        }
        result
    }

    pub fn channel_state_or_default(&self) -> Scalar {
        self.channels
            .get(0)
            .map(|channel| channel.state)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStackText {
    pub consume: bool,
    state: String,
}

impl InputStackText {
    pub fn state(&self) -> &str {
        &self.state
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputStackCombinationAction {
    Trigger(String),
    AxesMagnitude {
        name: String,
        threshold: Scalar,
    },
    AxesTargetValues {
        name: String,
        target: Vec<Scalar>,
        threshold: Scalar,
    },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputStackCombination {
    #[serde(default)]
    pub continous: bool,
    #[serde(default)]
    actions: Vec<InputStackCombinationAction>,
    #[serde(skip)]
    state: bool,
    #[serde(skip)]
    last_state: bool,
}

impl InputStackCombination {
    pub fn new(actions: impl Iterator<Item = impl Into<InputStackCombinationAction>>) -> Self {
        Self {
            continous: false,
            actions: actions.map(|action| action.into()).collect(),
            state: false,
            last_state: false,
        }
    }

    pub fn actions(&self) -> &[InputStackCombinationAction] {
        &self.actions
    }

    pub fn pressed(&self) -> bool {
        if self.continous {
            self.state
        } else {
            self.state && !self.last_state
        }
    }

    pub fn released(&self) -> bool {
        if self.continous {
            !self.state
        } else {
            !self.state && self.last_state
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputStackListener {
    #[serde(default)]
    pub priority: usize,
    #[serde(default = "InputStackListener::default_enabled")]
    pub enabled: bool,
    #[serde(skip)]
    pub bound_entity: Option<Entity>,
    #[serde(default)]
    triggers: HashMap<String, InputStackTrigger>,
    #[serde(default)]
    axes: HashMap<String, InputStackAxes>,
    #[serde(default)]
    text: Option<InputStackText>,
    #[serde(default)]
    combinations: HashMap<String, InputStackCombination>,
}

impl Default for InputStackListener {
    fn default() -> Self {
        Self {
            priority: 0,
            enabled: Self::default_enabled(),
            bound_entity: None,
            triggers: Default::default(),
            axes: Default::default(),
            text: None,
            combinations: Default::default(),
        }
    }
}

impl InputStackListener {
    fn default_enabled() -> bool {
        true
    }

    pub fn with_trigger(mut self, name: &str, trigger: InputStackTrigger) -> Self {
        self.map_trigger(name, trigger);
        self
    }

    pub fn map_trigger(&mut self, name: &str, trigger: InputStackTrigger) {
        self.triggers.insert(name.to_owned(), trigger);
    }

    pub fn unmap_trigger(&mut self, name: &str) {
        self.triggers.remove(name);
    }

    pub fn trigger(&self, name: &str) -> Option<&InputStackTrigger> {
        self.triggers.get(name)
    }

    pub fn trigger_state_or_default(&self, name: &str) -> TriggerState {
        self.trigger(name)
            .map(|trigger| trigger.state())
            .unwrap_or_default()
    }

    pub fn with_axes(mut self, name: &str, axes: InputStackAxes) -> Self {
        self.map_axes(name, axes);
        self
    }

    pub fn map_axes(&mut self, name: &str, axes: InputStackAxes) {
        self.axes.insert(name.to_owned(), axes);
    }

    pub fn unmap_axes(&mut self, name: &str) {
        self.axes.remove(name);
    }

    pub fn axes(&self, name: &str) -> Option<&InputStackAxes> {
        self.axes.get(name)
    }

    pub fn axes_channels_or_default(&self, name: &str) -> &[InputStackChannel] {
        self.axes(name)
            .map(|axes| axes.channels())
            .unwrap_or_default()
    }

    pub fn axes_state_or_default<const N: usize>(&self, name: &str) -> [Scalar; N] {
        self.axes(name)
            .map(|axes| axes.channels_state_or_default())
            .unwrap_or_else(|| [0.0; N])
    }

    pub fn channel_state_or_default(&self, name: &str) -> Scalar {
        self.axes(name)
            .map(|axes| axes.channel_state_or_default())
            .unwrap_or_default()
    }

    pub fn with_text(mut self, text: InputStackText) -> Self {
        self.map_text(text);
        self
    }

    pub fn map_text(&mut self, text: InputStackText) {
        self.text = Some(text);
    }

    pub fn unmap_text(&mut self) {
        self.text = None;
    }

    pub fn text(&self) -> Option<&InputStackText> {
        self.text.as_ref()
    }

    pub fn text_state_or_default(&self) -> &str {
        self.text().map(|text| text.state()).unwrap_or_default()
    }

    pub fn with_combination(mut self, name: &str, combination: InputStackCombination) -> Self {
        self.map_combination(name, combination);
        self
    }

    pub fn map_combination(&mut self, name: &str, combination: InputStackCombination) {
        self.combinations.insert(name.to_owned(), combination);
    }

    pub fn unmap_combination(&mut self, name: &str) {
        self.combinations.remove(name);
    }

    pub fn combination(&self, name: &str) -> Option<&InputStackCombination> {
        self.combinations.get(name)
    }

    pub fn combination_pressed_or_default(&self, name: &str) -> bool {
        self.combination(name)
            .map(|combination| combination.pressed())
            .unwrap_or_default()
    }

    pub fn combination_released_or_default(&self, name: &str) -> bool {
        self.combination(name)
            .map(|combination| combination.released())
            .unwrap_or_default()
    }
}

#[derive(Default)]
pub struct InputStack {
    listeners: HashMap<InputStackListenerId, InputStackListener>,
}

impl InputStack {
    pub fn register(&mut self, listener: InputStackListener) -> InputStackListenerId {
        let id = InputStackListenerId::new();
        self.listeners.insert(id, listener);
        id
    }

    pub fn unregister(&mut self, id: InputStackListenerId) {
        self.listeners.remove(&id);
    }

    pub fn listeners(&self) -> impl Iterator<Item = &InputStackListener> {
        self.listeners.values()
    }

    pub fn listener(&self, id: InputStackListenerId) -> Option<&InputStackListener> {
        self.listeners.get(&id)
    }

    pub fn listener_by_instance(
        &self,
        instance: &InputStackInstance,
    ) -> Option<&InputStackListener> {
        instance.as_listener().and_then(|id| self.listener(id))
    }

    pub fn listeners_by_entity(&self, entity: Entity) -> impl Iterator<Item = &InputStackListener> {
        self.listeners()
            .filter(move |listener| listener.bound_entity == Some(entity))
    }

    pub fn process(&mut self, controller: &InputController, entity_changes: &EntityChanges) {
        self.listeners.retain(|_, listener| {
            listener
                .bound_entity
                .map(|entity| !entity_changes.has_despawned(entity))
                .unwrap_or(true)
        });

        let mut consumed_triggers = HashSet::with_capacity(controller.triggers().count());
        let mut consumed_axes = HashSet::with_capacity(controller.axes().count());
        let mut consumed_text = false;
        let mut stack = self.listeners.iter_mut().collect::<Vec<_>>();
        stack.sort_by(|a, b| a.1.priority.cmp(&b.1.priority).reverse());

        for (_, listener) in stack {
            for trigger in listener.triggers.values_mut() {
                trigger.state = trigger.state.release();
                if listener.enabled {
                    if let Some((mapping, state)) = trigger
                        .mappings
                        .iter()
                        .filter(|m| !consumed_triggers.contains(m.as_str()))
                        .map(|m| (m, controller.trigger_or_default(m)))
                        .max_by(|a, b| a.1.priority().cmp(&b.1.priority()))
                    {
                        trigger.state = state;
                        if trigger.consume {
                            consumed_triggers.insert(mapping.to_owned());
                        }
                    }
                }
            }
            for axes in listener.axes.values_mut() {
                for channel in &mut axes.channels {
                    channel.state = 0.0;
                    if listener.enabled {
                        if let Some((mapping, scale, value)) = channel
                            .mappings
                            .iter()
                            .filter(|(m, _)| !consumed_axes.contains(m.as_str()))
                            .map(|(m, s)| (m, s, controller.axis_or_default(m)))
                            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                        {
                            channel.state = value * scale;
                            if axes.consume {
                                consumed_axes.insert(mapping.to_owned());
                            }
                        }
                    }
                }
            }
            if let Some(text) = listener.text.as_mut() {
                if !listener.enabled || consumed_text {
                    text.state.clear();
                } else {
                    text.state = controller.text().to_owned();
                    if text.consume {
                        consumed_text = true;
                    }
                }
            }
            let mut combinations = std::mem::take(&mut listener.combinations);
            for combination in combinations.values_mut() {
                combination.last_state = combination.state;
                combination.state = combination
                    .actions
                    .iter()
                    .map(|action| match action {
                        InputStackCombinationAction::Trigger(name) => {
                            listener.trigger_state_or_default(name).is_pressed()
                        }
                        InputStackCombinationAction::AxesMagnitude { name, threshold } => {
                            let axes = listener.axes_channels_or_default(name);
                            let squared = axes
                                .iter()
                                .map(|channel| channel.state() * channel.state())
                                .sum::<Scalar>();
                            let magnitude = if !axes.is_empty() {
                                squared / axes.len() as Scalar
                            } else {
                                0.0
                            };
                            magnitude > *threshold
                        }
                        InputStackCombinationAction::AxesTargetValues {
                            name,
                            target,
                            threshold,
                        } => {
                            let axes = listener.axes_channels_or_default(name);
                            let squared = target
                                .iter()
                                .zip(axes.iter())
                                .map(|(target, channel)| {
                                    let diff = (target - channel.state()).abs();
                                    diff * diff
                                })
                                .sum::<Scalar>();
                            let difference = if !axes.is_empty() {
                                squared / axes.len() as Scalar
                            } else {
                                0.0
                            };
                            difference > *threshold
                        }
                    })
                    .all(|v| v);
            }
            listener.combinations = combinations;
        }
    }
}
