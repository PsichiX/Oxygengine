use crate::{
    component::{CompositeRenderable, CompositeSprite},
    composite_renderer::Image,
    math::Rect,
    sprite_sheet_asset_protocol::SpriteSheetAsset,
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeSpriteSheetSystemCache {
    images_cache: HashMap<String, String>,
    atlas_table: HashMap<AssetId, String>,
    frames_cache: HashMap<String, HashMap<String, Rect>>,
}

pub type CompositeSpriteSheetSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a mut CompositeSpriteSheetSystemCache,
    Comp<&'a mut CompositeRenderable>,
    Comp<&'a mut CompositeSprite>,
);

pub fn composite_sprite_sheet_system(universe: &mut Universe) {
    let (world, assets, mut cache, ..) =
        universe.query_resources::<CompositeSpriteSheetSystemResources>();

    for id in assets.lately_loaded_protocol("atlas") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded atlas asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<SpriteSheetAsset>()
            .expect("trying to use non-atlas asset");
        let image = asset.info().meta.image_name();
        let frames = asset
            .info()
            .frames
            .iter()
            .map(|(k, v)| (k.to_owned(), v.frame))
            .collect();
        cache.images_cache.insert(path.clone(), image);
        cache.atlas_table.insert(id, path.clone());
        cache.frames_cache.insert(path, frames);
    }
    for id in assets.lately_unloaded_protocol("atlas") {
        if let Some(path) = cache.atlas_table.remove(id) {
            cache.images_cache.remove(&path);
            cache.frames_cache.remove(&path);
        }
    }

    for (_, (renderable, sprite)) in world
        .query::<(&mut CompositeRenderable, &mut CompositeSprite)>()
        .iter()
    {
        if sprite.dirty {
            if let Some((sheet, frame)) = sprite.sheet_frame() {
                if let (Some(name), Some(frames)) =
                    (cache.images_cache.get(sheet), cache.frames_cache.get(sheet))
                {
                    renderable.0 = Image {
                        image: name.clone().into(),
                        source: frames.get(frame).copied(),
                        destination: None,
                        alignment: sprite.alignment,
                    }
                    .into();
                    sprite.dirty = false;
                }
            }
        }
    }
}
