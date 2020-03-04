use crate::{
    interface::{ResourceAccess, ResourceModify},
    scriptable::ScriptableValue,
};
use oxygengine_core::Scalar;
use oxygengine_input::resource::{InputController, TriggerState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputControllerMappingsScripted {
    pub mapping_axes: HashMap<String, (String, String)>,
    pub mapping_triggers: HashMap<String, (String, String)>,
}

impl From<&InputController> for InputControllerMappingsScripted {
    fn from(value: &InputController) -> Self {
        Self {
            mapping_axes: value
                .mapping_axes()
                .map(|(k, (a, b))| (k.to_owned(), (a.to_owned(), b.to_owned())))
                .collect::<HashMap<_, _>>(),
            mapping_triggers: value
                .mapping_triggers()
                .map(|(k, (a, b))| (k.to_owned(), (a.to_owned(), b.to_owned())))
                .collect::<HashMap<_, _>>(),
        }
    }
}

impl ResourceModify<InputControllerMappingsScripted> for InputController {
    fn modify_resource(&mut self, source: InputControllerMappingsScripted) {
        for (k, (a, b)) in source.mapping_axes {
            self.map_axis(&k, &a, &b);
        }
        for (k, (a, b)) in source.mapping_triggers {
            self.map_trigger(&k, &a, &b);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputControllerStateScripted {
    pub axes: HashMap<String, Scalar>,
    pub triggers: HashMap<String, TriggerState>,
}

impl From<&InputController> for InputControllerStateScripted {
    fn from(value: &InputController) -> Self {
        Self {
            axes: value
                .axes()
                .map(|(k, v)| (k.to_owned(), v))
                .collect::<HashMap<_, _>>(),
            triggers: value
                .triggers()
                .map(|(k, v)| (k.to_owned(), v))
                .collect::<HashMap<_, _>>(),
        }
    }
}

impl ResourceModify<InputControllerStateScripted> for InputController {
    fn modify_resource(&mut self, source: InputControllerStateScripted) {
        for (k, v) in source.axes {
            self.set_axis(&k, v);
        }
        for (k, v) in source.triggers {
            self.set_trigger(&k, v);
        }
    }
}

impl ResourceAccess for InputController {
    fn access_resource(&mut self, _value: ScriptableValue) -> ScriptableValue {
        ScriptableValue::Null
    }
}
