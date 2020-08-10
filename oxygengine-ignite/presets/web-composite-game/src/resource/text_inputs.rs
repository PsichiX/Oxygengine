use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct TextInputs {
    pub focused: Option<String>,
    pub inputs: HashMap<String, (String, usize)>,
}
