use crate::interface::ComponentModify;
use oxygengine_composite_renderer::{component::*, math::*};
use oxygengine_utils::grid_2d::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSurfaceCacheScripted {
    pub name: String,
    pub width: usize,
    pub height: usize,
}

impl From<CompositeSurfaceCache> for CompositeSurfaceCacheScripted {
    fn from(value: CompositeSurfaceCache) -> Self {
        Self {
            name: value.name().into(),
            width: value.width(),
            height: value.height(),
        }
    }
}

impl From<CompositeSurfaceCacheScripted> for CompositeSurfaceCache {
    fn from(value: CompositeSurfaceCacheScripted) -> Self {
        Self::new(value.name.into(), value.width, value.height)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeTransformScripted {
    pub translation: Vec2,
    pub rotation: Scalar,
    pub scale: Vec2,
}

impl From<CompositeTransform> for CompositeTransformScripted {
    fn from(value: CompositeTransform) -> Self {
        Self {
            translation: value.get_translation(),
            rotation: value.get_rotation(),
            scale: value.get_scale(),
        }
    }
}

impl From<CompositeTransformScripted> for CompositeTransform {
    fn from(value: CompositeTransformScripted) -> Self {
        Self::new(value.translation, value.rotation, value.scale)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeSpriteScripted {
    pub alignment: Vec2,
    pub sheet_frame: Option<(String, String)>,
}

impl From<CompositeSprite> for CompositeSpriteScripted {
    fn from(value: CompositeSprite) -> Self {
        Self {
            alignment: value.alignment,
            sheet_frame: value
                .sheet_frame()
                .map(|(s, f)| (s.to_owned(), f.to_owned())),
        }
    }
}

impl From<CompositeSpriteScripted> for CompositeSprite {
    fn from(value: CompositeSpriteScripted) -> Self {
        let mut r = Self::default().align(value.alignment);
        r.set_sheet_frame(value.sheet_frame.map(|(s, f)| (s.into(), f.into())));
        r
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSpriteAnimationScriptedCurrent {
    pub name: String,
    pub phase: Scalar,
    pub speed: Scalar,
    pub looped: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSpriteAnimationScripted {
    pub animations: HashMap<String, SpriteAnimation>,
    pub current: Option<CompositeSpriteAnimationScriptedCurrent>,
}

impl From<CompositeSpriteAnimation> for CompositeSpriteAnimationScripted {
    fn from(value: CompositeSpriteAnimation) -> Self {
        Self {
            animations: value
                .animations
                .iter()
                .map(|(k, v)| (k.clone().into(), v.clone()))
                .collect::<HashMap<_, _>>(),
            current: if value.is_playing() {
                Some(CompositeSpriteAnimationScriptedCurrent {
                    name: value.current().unwrap().to_owned(),
                    phase: value.phase().unwrap(),
                    speed: value.speed().unwrap(),
                    looped: value.looped().unwrap(),
                })
            } else {
                None
            },
        }
    }
}

impl ComponentModify<CompositeSpriteAnimationScripted> for CompositeSpriteAnimation {
    fn modify_component(&mut self, source: CompositeSpriteAnimationScripted) {
        self.animations = source
            .animations
            .iter()
            .map(|(k, v)| (k.clone().into(), v.clone()))
            .collect::<HashMap<_, _>>();
        if let Some(current) = source.current {
            self.play(&current.name, current.speed, current.looped);
            self.set_phase(current.phase);
        } else {
            self.stop();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTilemapScripted {
    pub tileset: Option<String>,
    pub grid: Grid2d<TileCell>,
}

impl From<CompositeTilemap> for CompositeTilemapScripted {
    fn from(value: CompositeTilemap) -> Self {
        Self {
            tileset: value.tileset().map(|v| v.into()),
            grid: value.grid().clone(),
        }
    }
}

impl From<CompositeTilemapScripted> for CompositeTilemap {
    fn from(value: CompositeTilemapScripted) -> Self {
        let mut r = Self::default();
        r.set_tileset(value.tileset.map(|v| v.into()));
        r.set_grid(value.grid.clone());
        r
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemapAnimationScriptedCurrent {
    pub name: String,
    pub phase: Scalar,
    pub speed: Scalar,
    pub looped: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemapAnimationScripted {
    pub animations: HashMap<String, TilemapAnimation>,
    pub current: Option<CompositeTilemapAnimationScriptedCurrent>,
}

impl From<CompositeTilemapAnimation> for CompositeTilemapAnimationScripted {
    fn from(value: CompositeTilemapAnimation) -> Self {
        Self {
            animations: value
                .animations
                .iter()
                .map(|(k, v)| (k.clone().into(), v.clone()))
                .collect::<HashMap<_, _>>(),
            current: if value.is_playing() {
                Some(CompositeTilemapAnimationScriptedCurrent {
                    name: value.current().unwrap().to_owned(),
                    phase: value.phase().unwrap(),
                    speed: value.speed().unwrap(),
                    looped: value.looped().unwrap(),
                })
            } else {
                None
            },
        }
    }
}

impl ComponentModify<CompositeTilemapAnimationScripted> for CompositeTilemapAnimation {
    fn modify_component(&mut self, source: CompositeTilemapAnimationScripted) {
        self.animations = source
            .animations
            .iter()
            .map(|(k, v)| (k.clone().into(), v.clone()))
            .collect::<HashMap<_, _>>();
        if let Some(current) = source.current {
            self.play(&current.name, current.speed, current.looped);
            self.set_phase(current.phase);
        } else {
            self.stop();
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeMapChunkScripted {
    pub map_name: String,
    pub layer_name: String,
    pub offset: (usize, usize),
    pub size: Option<(usize, usize)>,
}

impl From<CompositeMapChunk> for CompositeMapChunkScripted {
    fn from(value: CompositeMapChunk) -> Self {
        Self {
            map_name: value.map_name().to_owned(),
            layer_name: value.layer_name().to_owned(),
            offset: value.offset(),
            size: value.size(),
        }
    }
}

impl From<CompositeMapChunkScripted> for CompositeMapChunk {
    fn from(value: CompositeMapChunkScripted) -> Self {
        let mut r = Self::new(value.map_name.into(), value.layer_name.into());
        r.set_offset(value.offset);
        r.set_size(value.size);
        r
    }
}
