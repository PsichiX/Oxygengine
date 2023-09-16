use core::scripting::intuicio::core::{function::FunctionHandle, struct_type::StructHandle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RigControl {
    pub struct_type: StructHandle,
    pub init_function: Option<FunctionHandle>,
    pub solve_function: FunctionHandle,
    /// {field name: field string value}
    pub bindings: HashMap<String, String>,
}
