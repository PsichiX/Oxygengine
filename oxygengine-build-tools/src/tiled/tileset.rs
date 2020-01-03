use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledTile {
    pub id: usize,
    pub image: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledTileset {
    #[serde(alias = "tilewidth")]
    pub tile_width: usize,
    #[serde(alias = "tileheight")]
    pub tile_height: usize,
    pub tiles: Vec<TiledTile>,
}
