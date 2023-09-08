use core::scripting::intuicio::core::{function::FunctionHandle, struct_type::StructHandle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RigControl {
    pub struct_type: StructHandle,
    pub function: FunctionHandle,
    /// {field name: field string value}
    pub bindings: HashMap<String, String>,
}
