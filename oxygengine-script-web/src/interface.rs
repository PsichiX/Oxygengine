use js_sys::Function;
use std::{collections::HashMap, sync::Mutex};
use wasm_bindgen::JsValue;

lazy_static! {
    static ref INTERFACE: Mutex<OxygenInterface> = Mutex::new(OxygenInterface::default());
}

#[derive(Default)]
pub struct OxygenInterface {
    // component_factory: HashMap<String, Function>,
    state_factory: HashMap<String, Function>,
}

unsafe impl Send for OxygenInterface {}
unsafe impl Sync for OxygenInterface {}

impl OxygenInterface {
    pub fn register_state_factory(name: &str, factory: Function) {
        if let Ok(mut interface) = INTERFACE.lock() {
            interface.state_factory.insert(name.to_owned(), factory);
        }
    }

    pub fn build_state(name: &str) -> Option<JsValue> {
        if let Ok(interface) = INTERFACE.lock() {
            if let Some(factory) = interface.state_factory.get(name) {
                if let Ok(result) = factory.call0(&JsValue::UNDEFINED) {
                    return Some(result);
                }
            }
        }
        None
    }
}
