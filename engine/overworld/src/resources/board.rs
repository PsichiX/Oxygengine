use oxygengine_core::{id::ID, Ignite};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

pub type BoardToken = ID<Location>;

#[derive(Debug, Clone)]
pub enum BoardError {
    ChunkAlreadyExists(BoardLocation),
    ChunkDoesNotExists(BoardLocation),
    ChunkLocationOutOfBounds(ChunkLocation),
    LocationOccupied(ChunkLocation),
    LocationUnoccupied(ChunkLocation),
    LocationUnavailable(ChunkLocation),
    TokenDoesNotExists(BoardToken),
    CannotTraverse(usize, usize),
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl BoardDirection {
    pub fn from_coord(mut x: isize, mut y: isize) -> Option<Self> {
        x = x.max(-1).min(1);
        y = y.max(-1).min(1);
        match (x, y) {
            (0, -1) => Some(Self::North),
            (1, -1) => Some(Self::NorthEast),
            (1, 0) => Some(Self::East),
            (1, 1) => Some(Self::SouthEast),
            (0, 1) => Some(Self::South),
            (-1, 1) => Some(Self::SouthWest),
            (-1, 0) => Some(Self::West),
            (-1, -1) => Some(Self::West),
            _ => None,
        }
    }

    pub fn into_coords(self) -> (isize, isize) {
        match self {
            Self::North => (0, -1),
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (1, 1),
            Self::South => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (-1, -1),
        }
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardCollision {
    #[serde(default)]
    pub token: Option<BoardToken>,
    #[serde(default)]
    pub traverse_block: Option<usize>,
    #[serde(default)]
    pub tile_block: bool,
}

impl BoardCollision {
    pub fn is_any(&self) -> bool {
        self.token.is_some() || self.traverse_block.is_some() || self.tile_block
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoardLocation {
    pub col: isize,
    pub row: isize,
}

impl BoardLocation {
    pub fn as_tuple(&self) -> (isize, isize) {
        (self.col, self.row)
    }

    pub fn as_array(&self) -> [isize; 2] {
        [self.col, self.row]
    }
}

impl From<(isize, isize)> for BoardLocation {
    fn from((col, row): (isize, isize)) -> Self {
        Self { col, row }
    }
}

impl From<[isize; 2]> for BoardLocation {
    fn from([col, row]: [isize; 2]) -> Self {
        Self { col, row }
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkLocation {
    pub col: usize,
    pub row: usize,
}

impl ChunkLocation {
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.col, self.row)
    }

    pub fn as_array(&self) -> [usize; 2] {
        [self.col, self.row]
    }
}

impl From<(usize, usize)> for ChunkLocation {
    fn from((col, row): (usize, usize)) -> Self {
        Self { col, row }
    }
}

impl From<[usize; 2]> for ChunkLocation {
    fn from([col, row]: [usize; 2]) -> Self {
        Self { col, row }
    }
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub world: BoardLocation,
    pub chunk: ChunkLocation,
}

impl From<(BoardLocation, ChunkLocation)> for Location {
    fn from((world, chunk): (BoardLocation, ChunkLocation)) -> Self {
        Self { world, chunk }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoardTraverseRules {
    map: HashMap<usize, HashSet<usize>>,
}

impl BoardTraverseRules {
    pub fn add(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        self.map.entry(from).or_default().insert(to);
    }

    pub fn with(mut self, from: usize, to: usize) -> Self {
        self.add(from, to);
        self
    }

    pub fn add_both_ways(&mut self, from: usize, to: usize) {
        self.add(from, to);
        self.add(to, from);
    }

    pub fn with_both_ways(mut self, from: usize, to: usize) -> Self {
        self.add_both_ways(from, to);
        self
    }

    pub fn add_product(&mut self, values: &[usize]) {
        for a in values {
            for b in values {
                self.add(*a, *b);
            }
        }
    }

    pub fn with_product(mut self, values: &[usize]) -> Self {
        self.add_product(values);
        self
    }

    pub fn remove(&mut self, from: usize, to: Option<usize>) {
        if let Some(to) = to {
            if let Some(values) = self.map.get_mut(&from) {
                values.remove(&to);
                if values.is_empty() {
                    self.map.remove(&from);
                }
            }
        } else if self.map.remove(&from).is_some() {
            for values in self.map.values_mut() {
                values.remove(&from);
            }
        }
    }

    pub fn can_traverse(&self, from: usize, to: usize) -> bool {
        from == to
            || self
                .map
                .get(&from)
                .map(|values| values.contains(&to))
                .unwrap_or_default()
    }

    pub fn can_sequence_traverse(&self, iter: impl Iterator<Item = usize>) -> bool {
        let mut last = None;
        for next in iter {
            let value = last.unwrap_or(next);
            if !self.can_traverse(value, next) {
                return false;
            }
            last = Some(next);
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct BoardChunk {
    cols: usize,
    rows: usize,
    tile_values: Vec<Option<usize>>,
    occupancy: Vec<Option<BoardToken>>,
    tokens: HashMap<BoardToken, ChunkLocation>,
}

impl BoardChunk {
    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    pub fn read_values(&self) -> &[Option<usize>] {
        &self.tile_values
    }

    pub fn write_values(&mut self) -> &mut [Option<usize>] {
        &mut self.tile_values
    }

    pub fn tile_value(&self, location: ChunkLocation) -> Result<Option<usize>, BoardError> {
        if location.col >= self.cols || location.row >= self.rows {
            return Err(BoardError::ChunkLocationOutOfBounds(location));
        }
        let index = location.row * self.cols + location.col;
        Ok(self.tile_values[index])
    }

    pub fn occupancy(&self, location: ChunkLocation) -> Result<Option<BoardToken>, BoardError> {
        if location.col >= self.cols || location.row >= self.rows {
            return Err(BoardError::ChunkLocationOutOfBounds(location));
        }
        let index = location.row * self.cols + location.col;
        Ok(self.occupancy[index])
    }

    pub fn token_location(&self, token: BoardToken) -> Option<ChunkLocation> {
        self.tokens.get(&token).cloned()
    }

    pub fn tokens(&self) -> impl Iterator<Item = (BoardToken, ChunkLocation)> + '_ {
        self.tokens
            .iter()
            .map(|(token, location)| (*token, *location))
    }

    fn acquire_token(&mut self, location: ChunkLocation) -> Result<BoardToken, BoardError> {
        if location.col >= self.cols || location.row >= self.rows {
            return Err(BoardError::ChunkLocationOutOfBounds(location));
        }
        let index = location.row * self.cols + location.col;
        if self.tile_values[index].is_none() {
            return Err(BoardError::LocationUnavailable(location));
        }
        if self.occupancy[index].is_some() {
            return Err(BoardError::LocationOccupied(location));
        }
        let token = BoardToken::new();
        self.occupancy[index] = Some(token);
        self.tokens.insert(token, location);
        Ok(token)
    }

    fn release_token(&mut self, token: BoardToken) -> Option<ChunkLocation> {
        if let Some(location) = self.tokens.remove(&token) {
            let index = location.row * self.cols + location.col;
            self.occupancy[index] = None;
            return Some(location);
        }
        None
    }

    fn occupy_location(
        &mut self,
        location: ChunkLocation,
        token: BoardToken,
    ) -> Result<(), BoardError> {
        if location.col >= self.cols || location.row >= self.rows {
            return Err(BoardError::ChunkLocationOutOfBounds(location));
        }
        let index = location.row * self.cols + location.col;
        if self.tile_values[index].is_none() {
            return Err(BoardError::LocationUnavailable(location));
        }
        if self.occupancy[index].is_some() {
            return Err(BoardError::LocationOccupied(location));
        }
        self.occupancy[index] = Some(token);
        self.tokens.insert(token, location);
        Ok(())
    }

    fn free_location(&mut self, location: ChunkLocation) -> Result<(), BoardError> {
        if location.col >= self.cols || location.row >= self.rows {
            return Err(BoardError::ChunkLocationOutOfBounds(location));
        }
        let index = location.row * self.cols + location.col;
        if let Some(token) = self.occupancy[index] {
            self.occupancy[index] = None;
            self.tokens.remove(&token);
            return Ok(());
        }
        Err(BoardError::LocationUnoccupied(location))
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    chunk_cols: usize,
    chunk_rows: usize,
    pub traverse_rules: BoardTraverseRules,
    chunks: HashMap<BoardLocation, BoardChunk>,
}

impl Board {
    pub fn new(chunk_cols: usize, chunk_rows: usize, traverse_rules: BoardTraverseRules) -> Self {
        Self {
            chunk_cols: chunk_cols.max(1),
            chunk_rows: chunk_rows.max(1),
            traverse_rules,
            chunks: Default::default(),
        }
    }

    /// (cols, rows)
    pub fn chunk_size(&self) -> (usize, usize) {
        (self.chunk_cols, self.chunk_rows)
    }

    pub fn location_move(&self, mut location: Location, x: isize, y: isize) -> Location {
        match x.cmp(&0) {
            Ordering::Greater => {
                location.chunk.col += x as usize;
                if location.chunk.col >= self.chunk_cols {
                    let times = location.chunk.col / self.chunk_cols;
                    location.world.col += times as isize;
                    location.chunk.col %= self.chunk_cols;
                }
            }
            Ordering::Less => {
                let v = location.chunk.col as isize + x;
                if v < 0 {
                    let v = -v;
                    let times = 1 + v / self.chunk_cols as isize;
                    location.world.col -= times;
                    location.chunk.col = self.chunk_cols - (v as usize % self.chunk_cols);
                } else {
                    location.chunk.col = v as usize;
                }
            }
            _ => {}
        }
        match y.cmp(&0) {
            Ordering::Greater => {
                location.chunk.row += y as usize;
                if location.chunk.row >= self.chunk_rows {
                    let times = location.chunk.row / self.chunk_rows;
                    location.world.row += times as isize;
                    location.chunk.row %= self.chunk_rows;
                }
            }
            Ordering::Less => {
                let v = location.chunk.row as isize + y;
                if v < 0 {
                    let v = -v;
                    let times = 1 + v / self.chunk_rows as isize;
                    location.world.row -= times;
                    location.chunk.row = self.chunk_rows - (v as usize % self.chunk_rows);
                } else {
                    location.chunk.row = v as usize;
                }
            }
            _ => {}
        }
        location
    }

    pub fn locations_in_range(&self, a: Location, b: Location, range: usize) -> bool {
        let dw = b.world.col - a.world.col;
        match dw.cmp(&0) {
            Ordering::Greater => {
                let dc = dw as usize * self.chunk_cols + b.chunk.col - a.chunk.col;
                if dc > range {
                    return false;
                }
            }
            Ordering::Less => {
                let dc = (-dw) as usize * self.chunk_cols + a.chunk.col - b.chunk.col;
                if dc > range {
                    return false;
                }
            }
            _ => {}
        }
        let dh = b.world.row - a.world.row;
        match dh.cmp(&0) {
            Ordering::Greater => {
                let dc = dh as usize * self.chunk_rows + b.chunk.row - a.chunk.row;
                if dc > range {
                    return false;
                }
            }
            Ordering::Less => {
                let dc = (-dh) as usize * self.chunk_rows + a.chunk.row - b.chunk.row;
                if dc > range {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    pub fn create_chunk(&mut self, location: BoardLocation) -> Result<(), BoardError> {
        if self.chunks.contains_key(&location) {
            return Err(BoardError::ChunkAlreadyExists(location));
        }
        let size = self.chunk_cols * self.chunk_rows;
        self.chunks.insert(
            location,
            BoardChunk {
                cols: self.chunk_cols,
                rows: self.chunk_rows,
                tile_values: vec![None; size],
                tokens: Default::default(),
                occupancy: vec![None; size],
            },
        );
        Ok(())
    }

    pub fn ensure_chunk(&mut self, location: BoardLocation) -> Result<(), BoardError> {
        if !self.chunks.contains_key(&location) {
            return self.create_chunk(location);
        }
        Ok(())
    }

    pub fn destroy_chunk(&mut self, location: BoardLocation) -> Result<(), BoardError> {
        if self.chunks.remove(&location).is_some() {
            Ok(())
        } else {
            Err(BoardError::ChunkDoesNotExists(location))
        }
    }

    pub fn has_chunk(&self, location: BoardLocation) -> bool {
        self.chunks.contains_key(&location)
    }

    pub fn chunk(&self, location: BoardLocation) -> Option<&BoardChunk> {
        self.chunks.get(&location)
    }

    pub fn chunk_mut(&mut self, location: BoardLocation) -> Option<&mut BoardChunk> {
        self.chunks.get_mut(&location)
    }

    pub fn tile_value(&self, location: Location) -> Result<Option<usize>, BoardError> {
        match self.chunk(location.world) {
            Some(chunk) => chunk.tile_value(location.chunk),
            None => Err(BoardError::ChunkDoesNotExists(location.world)),
        }
    }

    pub fn occupancy(&self, location: Location) -> Result<Option<BoardToken>, BoardError> {
        match self.chunk(location.world) {
            Some(chunk) => chunk.occupancy(location.chunk),
            None => Err(BoardError::ChunkDoesNotExists(location.world)),
        }
    }

    pub fn token_location(&self, token: BoardToken) -> Option<Location> {
        self.chunks
            .iter()
            .find_map(|(wloc, chunk)| Some((*wloc, chunk.token_location(token)?).into()))
    }

    pub fn tokens(&self) -> impl Iterator<Item = (BoardToken, Location)> + '_ {
        self.chunks.iter().flat_map(|(wloc, chunk)| {
            chunk
                .tokens()
                .map(|(token, cloc)| (token, (*wloc, cloc).into()))
        })
    }

    pub fn tokens_in_range(
        &self,
        location: Location,
        range: usize,
    ) -> impl Iterator<Item = (BoardToken, Location)> + '_ {
        self.tokens()
            .filter(move |(_, loc)| self.locations_in_range(location, *loc, range))
    }

    pub fn tile_collision(
        &self,
        location: Location,
        direction: BoardDirection,
    ) -> Result<(Location, BoardCollision), BoardError> {
        let tile_value = match self.tile_value(location)? {
            Some(value) => value,
            None => return Err(BoardError::LocationUnavailable(location.chunk)),
        };
        let (x, y) = direction.into_coords();
        let loc = self.location_move(location, x, y);
        let value = self.tile_value(loc)?;
        let token = self.occupancy(loc)?;
        let tile_block = value.is_none();
        let traverse_block = value.and_then(|value| {
            if self.traverse_rules.can_traverse(tile_value, value) {
                Some(value)
            } else {
                None
            }
        });
        let collision = BoardCollision {
            token,
            traverse_block,
            tile_block,
        };
        Ok((loc, collision))
    }

    pub fn tile_collisions(
        &self,
        location: Location,
        include_diagonals: bool,
    ) -> impl Iterator<Item = (BoardDirection, Location, BoardCollision)> + '_ {
        let directions = if include_diagonals {
            &[
                BoardDirection::North,
                BoardDirection::NorthEast,
                BoardDirection::East,
                BoardDirection::SouthEast,
                BoardDirection::South,
                BoardDirection::SouthWest,
                BoardDirection::West,
                BoardDirection::NorthWest,
            ][..]
        } else {
            &[
                BoardDirection::North,
                BoardDirection::East,
                BoardDirection::South,
                BoardDirection::West,
            ][..]
        };
        directions
            .iter()
            .filter_map(move |direction| {
                Some((
                    *direction,
                    self.tile_collision(location, *direction)
                        .ok()
                        .filter(|(_, collision)| collision.is_any())?,
                ))
            })
            .map(|(direction, (location, collision))| (direction, location, collision))
    }

    pub fn acquire_token(&mut self, location: Location) -> Result<BoardToken, BoardError> {
        match self.chunks.get_mut(&location.world) {
            Some(chunk) => chunk.acquire_token(location.chunk),
            None => Err(BoardError::ChunkDoesNotExists(location.world)),
        }
    }

    pub fn release_token(&mut self, token: BoardToken) -> Option<Location> {
        for (wloc, chunk) in &mut self.chunks {
            if let Some(cloc) = chunk.release_token(token) {
                return Some((*wloc, cloc).into());
            }
        }
        None
    }

    pub fn move_token(&mut self, token: BoardToken, x: isize, y: isize) -> Result<(), BoardError> {
        if x == 0 && y == 0 {
            return Ok(());
        }
        let from = match self.token_location(token) {
            Some(from) => from,
            None => return Err(BoardError::TokenDoesNotExists(token)),
        };
        let to = self.location_move(from, x, y);
        if self.occupancy(to)?.is_some() {
            return Err(BoardError::LocationOccupied(to.chunk));
        }
        let from_value = match self.tile_value(from)? {
            Some(value) => value,
            None => return Err(BoardError::LocationUnavailable(to.chunk)),
        };
        let to_value = match self.tile_value(to)? {
            Some(value) => value,
            None => return Err(BoardError::LocationUnavailable(to.chunk)),
        };
        if !self.traverse_rules.can_traverse(from_value, to_value) {
            return Err(BoardError::CannotTraverse(from_value, to_value));
        }
        // NOTE: order matters! do not change it.
        self.free_location(from)?;
        self.occupy_location(to, token)?;
        Ok(())
    }

    pub fn move_step_token(
        &mut self,
        token: BoardToken,
        direction: BoardDirection,
    ) -> Result<(), BoardError> {
        let (x, y) = direction.into_coords();
        self.move_token(token, x, y)
    }

    pub fn move_sequence_token(
        &mut self,
        token: BoardToken,
        iter: impl Iterator<Item = BoardDirection>,
    ) -> Result<(), BoardError> {
        for direction in iter {
            self.move_step_token(token, direction)?;
        }
        Ok(())
    }

    pub fn teleport_token(&mut self, token: BoardToken, to: Location) -> Result<(), BoardError> {
        let from = match self.token_location(token) {
            Some(from) => from,
            None => return Err(BoardError::TokenDoesNotExists(token)),
        };
        if from == to {
            return Ok(());
        }
        // NOTE: order matters! do not change it.
        self.free_location(from)?;
        self.occupy_location(to, token)?;
        Ok(())
    }

    pub fn swap_tokens(&mut self, a: BoardToken, b: BoardToken) -> Result<(), BoardError> {
        if a == b {
            return Ok(());
        }
        let from = match self.token_location(a) {
            Some(from) => from,
            None => return Err(BoardError::TokenDoesNotExists(a)),
        };
        let to = match self.token_location(b) {
            Some(to) => to,
            None => return Err(BoardError::TokenDoesNotExists(b)),
        };
        self.free_location(from)?;
        self.free_location(to)?;
        self.occupy_location(from, b)?;
        self.occupy_location(to, a)?;
        Ok(())
    }

    fn occupy_location(&mut self, location: Location, token: BoardToken) -> Result<(), BoardError> {
        match self.chunks.get_mut(&location.world) {
            Some(chunk) => chunk.occupy_location(location.chunk, token),
            None => Err(BoardError::ChunkDoesNotExists(location.world)),
        }
    }

    fn free_location(&mut self, location: Location) -> Result<(), BoardError> {
        match self.chunks.get_mut(&location.world) {
            Some(chunk) => chunk.free_location(location.chunk),
            None => Err(BoardError::ChunkDoesNotExists(location.world)),
        }
    }
}
