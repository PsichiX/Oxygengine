use crate::{
    map::{TiledMap, TiledMapLayerType},
    tileset::TiledTileset,
};
use oxygengine_composite_renderer::sprite_sheet_asset_protocol::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::read_to_string,
    io::{Error, ErrorKind},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayerObject {
    pub name: String,
    pub object_type: String,
    pub visible: bool,
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LayerData {
    Tiles(Vec<usize>),
    Objects(Vec<LayerObject>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Layer {
    pub name: String,
    pub data: LayerData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub sprite_sheets: Vec<String>,
    pub tiles_mapping: HashMap<usize, (String, String)>,
    pub layers: Vec<Layer>,
}

pub fn build_map<P: AsRef<Path>>(
    input: P,
    spritesheets: &[P],
    full_names: bool,
) -> Result<Vec<u8>, Error> {
    println!("* Load Tiled map file: {:?}", input.as_ref());
    let input_json = read_to_string(input.as_ref())?;
    let input_data = match serde_json::from_str::<TiledMap>(&input_json) {
        Ok(data) => data,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Could not parse JSON input file: {:?}. Error: {:?}",
                    input.as_ref(),
                    error
                ),
            ))
        }
    };
    let tilesets_image_map = input_data
        .tilesets
        .iter()
        .map(|t| {
            println!("* Load Tiled tileset file: {:?}", &t.source);
            let mut path = input.as_ref().to_path_buf();
            path.pop();
            let path = path.join(&t.source);
            let json = read_to_string(&path)?;
            match serde_json::from_str::<TiledTileset>(&json) {
                Ok(data) => Ok((
                    t.firstgid,
                    data.tiles
                        .iter()
                        .map(|t| {
                            let name = if full_names {
                                t.image.to_str().unwrap().to_owned()
                            } else {
                                t.image.file_name().unwrap().to_str().unwrap().to_owned()
                            };
                            (t.id, name)
                        })
                        .collect::<Vec<_>>(),
                )),
                Err(error) => Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "Could not parse JSON tileset file: {:?}. Error: {:?}",
                        &path, error
                    ),
                )),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let image_id_map = tilesets_image_map
        .into_iter()
        .flat_map(|(fid, ts)| {
            ts.into_iter()
                .map(|(id, img)| (fid + id, img))
                .collect::<Vec<_>>()
        })
        .collect::<HashMap<_, _>>();
    let spritesheets_data = spritesheets
        .iter()
        .map(|s| {
            println!("* Load sprite sheet file: {:?}", s.as_ref());
            let name = if full_names {
                s.as_ref().to_str().unwrap().to_owned()
            } else {
                s.as_ref().file_name().unwrap().to_str().unwrap().to_owned()
            };
            let json = read_to_string(s.as_ref())?;
            match serde_json::from_str::<SpriteSheetInfo>(&json) {
                Ok(data) => Ok((name, data)),
                Err(error) => Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "Could not parse JSON spritesheet file: {:?}. Error: {:?}",
                        s.as_ref(),
                        error
                    ),
                )),
            }
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let layers = input_data
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let data = match layer.layer_type {
                TiledMapLayerType::TileLayer => {
                    if let Some(data) = &layer.data {
                        Ok(LayerData::Tiles(data.clone()))
                    } else {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("There is no tiles data for layer: {}", i),
                        ))
                    }
                }
                TiledMapLayerType::ObjectGroup => {
                    if let Some(objects) = &layer.objects {
                        Ok(LayerData::Objects(
                            objects
                                .iter()
                                .map(|o| LayerObject {
                                    name: o.name.clone(),
                                    object_type: o.object_type.clone(),
                                    visible: o.visible,
                                    x: o.x,
                                    y: o.y,
                                    width: o.width,
                                    height: o.height,
                                })
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("There is no objects data for layer: {}", i),
                        ))
                    }
                }
            }?;
            Ok(Layer {
                name: layer.name.clone(),
                data,
            })
        })
        .collect::<Result<Vec<Layer>, Error>>()?;
    let tiles_mapping = image_id_map
        .into_iter()
        .filter(|(id, _)| {
            layers.iter().any(|layer| {
                if let LayerData::Tiles(data) = &layer.data {
                    data.iter().any(|i| i == id)
                } else {
                    false
                }
            })
        })
        .map(|(id, img)| {
            if let Some((atl, _)) = spritesheets_data
                .iter()
                .find(|(_, s)| s.frames.keys().any(|k| k == &img))
            {
                Ok((id, (atl.clone(), img)))
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("Could not find image in spritesheets: {:?}", img),
                ))
            }
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let map = Map {
        cols: input_data.width,
        rows: input_data.height,
        tile_width: input_data.tilewidth,
        tile_height: input_data.tileheight,
        sprite_sheets: spritesheets_data.keys().cloned().collect::<Vec<_>>(),
        tiles_mapping,
        layers,
    };
    match bincode::serialize(&map) {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(Error::new(
            ErrorKind::Other,
            format!("Could not serialize map data: {:?}", error),
        )),
    }
}
