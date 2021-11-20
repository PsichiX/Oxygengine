use oxygengine_ha_renderer::math::Vec2;
use oxygengine_overworld::resources::board::BoardLocation;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct HaBoardSettings {
    cell_size: Vec2,
    origin: BoardLocation,
    valid_tile_values: HashSet<usize>,
    regions: HashSet<(BoardLocation, BoardLocation)>,
    dirty: bool,
}

impl HaBoardSettings {
    pub fn new(cell_size: Vec2) -> Self {
        Self {
            cell_size,
            origin: Default::default(),
            valid_tile_values: Default::default(),
            regions: Default::default(),
            dirty: true,
        }
    }

    pub fn cell_size(&self) -> Vec2 {
        self.cell_size
    }

    pub fn set_cell_size(&mut self, size: Vec2) {
        if size != self.cell_size {
            self.cell_size = size;
            self.dirty = true;
        }
    }

    pub fn origin(&self) -> BoardLocation {
        self.origin
    }

    pub fn set_origin(&mut self, location: BoardLocation) {
        if location != self.origin {
            self.origin = location;
            self.dirty = true;
        }
    }

    pub fn with_origin(mut self, location: BoardLocation) -> Self {
        self.set_origin(location);
        self
    }

    pub fn is_tile_value_valid(&self, value: usize) -> bool {
        self.valid_tile_values.contains(&value)
    }

    pub fn valid_tile_values(&self) -> impl Iterator<Item = usize> + '_ {
        self.valid_tile_values.iter().copied()
    }

    pub fn set_valid_tile_values(&mut self, iter: impl Iterator<Item = usize>) {
        self.valid_tile_values = iter.collect();
        self.dirty = true;
    }

    pub fn with_valid_tile_values(mut self, iter: impl Iterator<Item = usize>) -> Self {
        self.set_valid_tile_values(iter);
        self
    }

    /// (top left, bottom right)
    pub fn find_region(&self, location: BoardLocation) -> Option<(BoardLocation, BoardLocation)> {
        self.regions
            .iter()
            .find(|(tl, br)| {
                location.col >= tl.col
                    && location.col <= br.col
                    && location.row >= tl.row
                    && location.row <= br.row
            })
            .map(|(tl, br)| (*tl, *br))
    }

    pub fn add_region(&mut self, top_left: BoardLocation, bottom_right: BoardLocation) {
        self.regions.insert((top_left, bottom_right));
        self.dirty = true;
    }

    pub fn with_region(mut self, top_left: BoardLocation, bottom_right: BoardLocation) -> Self {
        self.add_region(top_left, bottom_right);
        self
    }

    pub fn remove_region(&mut self, top_left: BoardLocation, bottom_right: BoardLocation) {
        self.regions.remove(&(top_left, bottom_right));
        self.dirty = true;
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}
