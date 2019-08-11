#![allow(dead_code)]

pub struct HistoryRes {
    pub lines_limit: usize,
    text: String,
    dirty: bool,
}

impl Default for HistoryRes {
    fn default() -> Self {
        Self {
            lines_limit: 10,
            text: "".to_owned(),
            dirty: false,
        }
    }
}

impl HistoryRes {
    pub fn write(&mut self, msg: &str) {
        self.dirty = true;
        self.text = format!("> {}\n{}", msg, self.text)
            .lines()
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(format!("{}\n", line))
                }
            })
            .take(self.lines_limit)
            .collect();
    }

    pub fn read_text(&mut self) -> Option<&str> {
        if self.dirty {
            self.dirty = false;
            Some(&self.text)
        } else {
            None
        }
    }

    pub fn text(&mut self) -> &str {
        &self.text
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
