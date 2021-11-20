use crate::{components::HaChangeFrequency, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaTileMapTile {
    pub col: usize,
    pub row: usize,
    pub atlas_item: String,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaTileMapInstance {
    atlas: String,
    cols: usize,
    rows: usize,
    cell_size: Vec2,
    #[serde(default)]
    change_frequency: HaChangeFrequency,
    #[serde(default)]
    tiles: Vec<HaTileMapTile>,
    #[serde(default)]
    pivot: Vec2,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl Default for HaTileMapInstance {
    fn default() -> Self {
        Self {
            atlas: Default::default(),
            cols: 0,
            rows: 0,
            cell_size: Default::default(),
            change_frequency: Default::default(),
            tiles: Default::default(),
            pivot: Default::default(),
            dirty: true,
        }
    }
}

impl HaTileMapInstance {
    pub fn atlas(&self) -> &str {
        &self.atlas
    }

    pub fn set_atlas(&mut self, atlas: impl ToString) {
        self.atlas = atlas.to_string();
        self.dirty = true;
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn set_cols(&mut self, cols: usize) {
        self.cols = cols;
        self.dirty = true;
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn set_rows(&mut self, rows: usize) {
        self.rows = rows;
        self.dirty = true;
    }

    pub fn cell_size(&self) -> Vec2 {
        self.cell_size
    }

    pub fn set_cell_size(&mut self, cell_size: Vec2) {
        self.cell_size = cell_size;
        self.dirty = true;
    }

    pub fn change_frequency(&self) -> HaChangeFrequency {
        self.change_frequency
    }

    pub fn set_change_frequency(&mut self, frequency: HaChangeFrequency) {
        self.change_frequency = frequency;
        self.dirty = true;
    }

    pub fn tiles(&self) -> &[HaTileMapTile] {
        &self.tiles
    }

    pub fn set_tiles(&mut self, tiles: Vec<HaTileMapTile>) {
        self.tiles = tiles;
        self.dirty = true;
    }

    pub fn pivot(&self) -> Vec2 {
        self.pivot
    }

    pub fn set_pivot(&mut self, pivot: Vec2) {
        self.pivot = Vec2::partial_max(Vec2::partial_min(pivot, 1.0), 0.0);
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Prefab for HaTileMapInstance {
    fn post_from_prefab(&mut self) {
        self.pivot = Vec2::partial_max(Vec2::partial_min(self.pivot, 1.0), 0.0);
        self.dirty = true;
    }
}

impl PrefabComponent for HaTileMapInstance {}
