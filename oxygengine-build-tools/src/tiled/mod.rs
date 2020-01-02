use std::{
    collections::HashMap,
    fs::{read, write},
    io::{Error, ErrorKind},
    path::Path,
};
use serde::Serialize;

pub mod map;
pub mod tileset;

#[derive(Debug, Clone, Serialize)]
struct LayerObject {
    pub name: String,
    pub object_type: String,
    pub visible: bool,
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize)]
enum LayerData {
    Tiles(Vec<usize>),
    Objects(Vec<LayerObject>),
}

#[derive(Debug, Clone, Serialize)]
struct Layer {
    pub name: String,
    pub layer_type: String,
    pub data: LayerData,
}

#[derive(Debug, Clone, Serialize)]
struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub sprite_sheets: Vec<String>,
    pub tiles_mapping: HashMap<usize, String>,
    pub layers: Vec<Layer>,
}

// pub fn build_map<P: AsRef<Path>>(input: P, spritesheets: &[P]) -> Result<Map, Error> {
//
// }
