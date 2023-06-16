use crate::math::Rect;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vek::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformerArea {
    pub rectangle: Rect,
    pub cols: usize,
    pub rows: usize,
}

impl DeformerArea {
    /// (position, col, row, cell index)
    pub fn local_to_area(&self, mut position: Vec2<f32>) -> (Vec2<f32>, usize, usize, usize) {
        let cell_width = self.rectangle.w / self.cols as f32;
        let cell_height = self.rectangle.h / self.rows as f32;
        position.x = position
            .x
            .clamp(self.rectangle.x, self.rectangle.x + self.rectangle.w);
        position.y = position
            .y
            .clamp(self.rectangle.y, self.rectangle.y + self.rectangle.h);
        position.x -= self.rectangle.x;
        position.y -= self.rectangle.y;
        let mut col = (position.x / cell_width) as usize;
        let mut row = (position.y / cell_height) as usize;
        position.x = (position.x / cell_width).fract();
        position.y = (position.y / cell_height).fract();
        if col >= self.cols {
            position.x += 1.0;
            col = self.cols - 1;
        }
        if row >= self.rows {
            position.y += 1.0;
            row = self.rows - 1;
        }
        let index = row * self.cols + col;
        (position, col, row, index)
    }

    pub fn area_to_local(&self, mut position: Vec2<f32>, col: usize, row: usize) -> Vec2<f32> {
        let cell_width = self.rectangle.w / self.cols as f32;
        let cell_height = self.rectangle.h / self.rows as f32;
        position.x *= cell_width;
        position.y *= cell_height;
        position.x += self.rectangle.x + cell_width * col.min(self.cols) as f32;
        position.y += self.rectangle.y + cell_height * row.min(self.cols) as f32;
        position
    }

    pub fn cells_count(&self) -> usize {
        self.cols * self.rows
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Deformer {
    #[serde(default)]
    pub areas: BTreeMap<String, DeformerArea>,
}

impl Deformer {
    pub fn with_area(mut self, name: impl ToString, area: impl Into<DeformerArea>) -> Self {
        self.areas.insert(name.to_string(), area.into());
        self
    }

    pub fn find_area_cells_offset(&self, name: &str) -> Option<(DeformerArea, usize)> {
        let mut result = 0;
        for (n, area) in &self.areas {
            if n == name {
                return Some((area.to_owned(), result));
            }
            result += area.cells_count();
        }
        None
    }
}
