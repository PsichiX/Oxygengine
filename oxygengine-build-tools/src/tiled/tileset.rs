use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct TiledTile {
    pub id: usize,
    pub image: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledTileset {
    #[serde(alias = "tileWidth")]
    pub tile_width: usize,
    #[serde(alias = "tileHeight")]
    pub tile_height: usize,
    pub tiles: Vec<TiledTile>,
}
