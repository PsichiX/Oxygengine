use crate::{
    asset_protocols::atlas::AtlasAsset,
    ha_renderer::HaRenderer,
    image::{ImageResourceMapping, VirtualImage, VirtualImageId, VirtualImageSource},
    math::*,
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::Universe,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaAtlasSystemCache {
    atlas_map: HashMap<AssetId, HashMap<VirtualImageId, Vec<String>>>,
}

pub type HaAtlasSystemResources<'a> = (
    &'a mut HaRenderer,
    &'a AssetsDatabase,
    &'a mut HaAtlasSystemCache,
    &'a mut ImageResourceMapping,
);

pub fn ha_atlas_system(universe: &mut Universe) {
    let (mut renderer, assets, mut cache, mut image_mapping) =
        universe.query_resources::<HaAtlasSystemResources>();

    for id in assets.lately_loaded_protocol("atlas") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<AtlasAsset>() {
                for (page, mappings) in &asset.page_mappings {
                    if let Some((page_size, image_asset_id)) = asset.pages_image_assets.get(page) {
                        if let Some(image_id) = image_mapping.resource_by_asset(*image_asset_id) {
                            let virtual_image_id = renderer.virtual_images.add_named(
                                path.to_owned(),
                                VirtualImage::new(VirtualImageSource::Image(image_id)),
                            );
                            let virtual_images = cache.atlas_map.entry(*id).or_default();
                            let virtual_image =
                                renderer.virtual_images.get_mut(virtual_image_id).unwrap();
                            let mut subimages = Vec::with_capacity(mappings.len());
                            for (image, region) in mappings {
                                let uvs = rect(
                                    region.rect.x / page_size.x,
                                    region.rect.y / page_size.y,
                                    region.rect.w / page_size.x,
                                    region.rect.h / page_size.y,
                                );
                                let image_id = virtual_image.register_named_image_uvs(
                                    image,
                                    uvs,
                                    region.layer,
                                );
                                let name = format!("{}@{}", path, image);
                                subimages.push(name.to_owned());
                                image_mapping.map_virtual_resource(
                                    name,
                                    virtual_image_id,
                                    image_id,
                                );
                            }
                            virtual_images.insert(virtual_image_id, subimages);
                        }
                    }
                }
            }
        }
    }
    for id in assets.lately_unloaded_protocol("atlas") {
        if let Some(ids) = cache.atlas_map.remove(id) {
            for (id, names) in ids {
                renderer.virtual_images.remove(id);
                for name in names {
                    image_mapping.unmap_name(&name);
                }
            }
        }
    }
}
