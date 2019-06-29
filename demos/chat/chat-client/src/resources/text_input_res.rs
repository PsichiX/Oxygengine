#[derive(Default)]
pub struct TextInputRes {
    typing: String,
    dirty_typing: bool,
    values: Vec<String>,
    dirty_values: bool,
}

impl TextInputRes {
    pub fn push_typing(&mut self, character: char) {
        self.dirty_typing = true;
        self.typing.push(character);
    }

    pub fn pop_typing(&mut self) {
        self.dirty_typing = true;
        self.typing.pop();
    }

    pub fn read_typing(&mut self) -> Option<&str> {
        if self.dirty_typing {
            self.dirty_typing = false;
            Some(&self.typing)
        } else {
            None
        }
    }

    pub fn typing(&self) -> &str {
        &self.typing
    }

    pub fn store_typing(&mut self) {
        if self.typing.is_empty() {
            return;
        }
        self.dirty_typing = true;
        self.dirty_values = true;
        self.values.push(self.typing.clone());
        self.typing.clear();
    }

    pub fn is_dirty_typing(&self) -> bool {
        self.dirty_typing
    }

    pub fn read_values(&mut self) -> Option<Vec<String>> {
        if self.dirty_values {
            self.dirty_values = false;
            let result = self.values.clone();
            self.values.clear();
            Some(result)
        } else {
            None
        }
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub fn is_dirty_values(&self) -> bool {
        self.dirty_values
    }
}
