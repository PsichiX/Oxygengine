use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, Deserialize)]
pub(crate) enum TiledMapOrientation {
    #[serde(alias = "orthogonal")]
    Orthogonal,
    #[serde(alias = "isometric")]
    Isometric,
    #[serde(alias = "staggered")]
    Staggered,
    #[serde(alias = "hexagonal")]
    Hexagonal,
}

impl Default for TiledMapOrientation {
    fn default() -> Self {
        Self::Orthogonal
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub(crate) enum TiledMapRenderOrder {
    #[serde(alias = "right-down")]
    RightDown,
    #[serde(alias = "right-up")]
    RightUp,
    #[serde(alias = "left-down")]
    LeftDown,
    #[serde(alias = "left-up")]
    LeftUp,
}

impl Default for TiledMapRenderOrder {
    fn default() -> Self {
        Self::RightDown
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledMapTileset {
    pub firstgid: usize,
    pub source: PathBuf,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub(crate) enum TiledMapLayerType {
    #[serde(alias = "tilelayer")]
    TileLayer,
    #[serde(alias = "objectgroup")]
    ObjectGroup,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledMapObject {
    pub id: usize,
    #[serde(default)]
    pub name: String,
    #[serde(alias = "type")]
    #[serde(default)]
    pub object_type: String,
    #[serde(default)]
    pub visible: bool,
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledMapLayer {
    pub id: usize,
    pub name: String,
    pub visible: bool,
    #[serde(alias = "type")]
    pub layer_type: TiledMapLayerType,
    pub data: Option<Vec<usize>>,
    pub objects: Option<Vec<TiledMapObject>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiledMap {
    #[serde(default)]
    pub orientation: TiledMapOrientation,
    #[serde(default)]
    pub renderorder: TiledMapRenderOrder,
    pub width: usize,
    pub height: usize,
    pub tilewidth: usize,
    pub tileheight: usize,
    pub infinite: bool,
    pub tilesets: Vec<TiledMapTileset>,
    pub layers: Vec<TiledMapLayer>,
}
