use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub enum LevelError {
    /// (provided, expected)
    CellsStringSizeDoesNotMatchSize(usize, usize),
    UnsupportedObjectCharacter(char),
    UnsupportedTileCharacter(char),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LevelAsset {
    pub cols: usize,
    pub rows: usize,
    pub cells: String,
}

impl LevelAsset {
    pub fn build_data(&self) -> Result<LevelData, LevelError> {
        let cells = self
            .cells
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<Vec<_>>();
        let provided = cells.len();
        let expected = self.cols * self.rows * 2;
        if provided != expected {
            return Err(LevelError::CellsStringSizeDoesNotMatchSize(
                provided, expected,
            ));
        }
        Ok(LevelData {
            cols: self.cols,
            rows: self.rows,
            cells: cells
                .chunks(2)
                .map(|c| {
                    let object = match c[0] {
                        '.' => LevelObject::None,
                        '*' => LevelObject::Star,
                        '$' => LevelObject::Shield,
                        '@' => LevelObject::PlayerStart,
                        c => return Err(LevelError::UnsupportedObjectCharacter(c)),
                    };
                    let tile = match c[1] {
                        'o' => LevelTile::Ocean,
                        'g' => LevelTile::Grass,
                        'p' => LevelTile::Pond,
                        'l' => LevelTile::Lava,
                        c => return Err(LevelError::UnsupportedTileCharacter(c)),
                    };
                    Ok(LevelCell { tile, object })
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct LevelData {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<LevelCell>,
}

impl LevelData {
    pub fn build_render_commands(&self, cell_size: Scalar) -> Vec<Command<'static>> {
        self.cells
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let col = index % self.cols;
                let row = index / self.cols;
                Command::Draw(cell.tile.build_image(col, row, cell_size).into())
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub struct LevelCell {
    pub tile: LevelTile,
    pub object: LevelObject,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelTile {
    Ocean,
    Grass,
    Pond,
    Lava,
}

impl LevelTile {
    pub fn build_image(self, col: usize, row: usize, size: Scalar) -> Image<'static> {
        let image = match self {
            Self::Ocean => "images/tile-ocean.svg",
            Self::Grass => "images/tile-grass.svg",
            Self::Pond => "images/tile-pond.svg",
            Self::Lava => "images/tile-lava.svg",
        };
        Image::new(image).destination(Some(
            [size * col as Scalar, size * row as Scalar, size, size].into(),
        ))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelObject {
    None,
    Star,
    Shield,
    PlayerStart,
}
