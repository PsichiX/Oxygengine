use crate::{
    asset_protocols::skeleton::*,
    components::{material_instance::*, skeleton_instance::*},
    ha_renderer::HaRenderer,
    image::*,
    material::common::MaterialValue,
    mesh::skeleton::*,
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaSkinningSystemCache {
    map: HashMap<String, (Skeleton, AssetId)>,
    table: HashMap<AssetId, String>,
    data_images: HashMap<Entity, ImageId>,
}

pub type HaSkinningSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a EntityChanges,
    &'a mut HaRenderer,
    &'a mut HaSkinningSystemCache,
    Comp<&'a mut HaSkeletonInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_skinning_system(universe: &mut Universe) {
    let (world, assets, changes, mut renderer, mut cache, ..) =
        universe.query_resources::<HaSkinningSystemResources>();

    for id in assets.lately_loaded_protocol("skeleton") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(skeleton) = asset
                .get::<SkeletonAsset>()
                .map(|asset| asset.get())
                .and_then(|hierarchy| Skeleton::try_from(hierarchy.to_owned()).ok())
            {
                cache.map.insert(path.to_owned(), (skeleton, *id));
                cache.table.insert(*id, path.to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("skeleton") {
        if let Some(name) = cache.table.remove(id) {
            cache.map.remove(&name);
        }
    }
    for entity in changes.despawned() {
        if let Some(id) = cache.data_images.remove(&entity) {
            let _ = renderer.remove_image(id);
        }
    }

    for (entity, (skeleton, material)) in world
        .query::<(&mut HaSkeletonInstance, &mut HaMaterialInstance)>()
        .iter()
    {
        if skeleton.dirty {
            if let Some((asset, _)) = cache.map.get(skeleton.skeleton()) {
                skeleton.try_initialize_bone_transforms(asset);
                skeleton.recalculate_bone_matrices(asset);
                let image_id = cache.data_images.get(&entity).copied().or_else(|| {
                    let descriptor = ImageDescriptor {
                        mode: ImageMode::Image2d,
                        format: ImageFormat::Data,
                        mipmap: ImageMipmap::None,
                    };
                    let data = unsafe { [0.0_f32, 0.0, 0.0, 0.0].align_to::<u8>().1 }.to_vec();
                    let image = Image::new(descriptor, 1, 1, 1, data);
                    image.ok().and_then(|image| renderer.add_image(image).ok())
                });
                if let Some(id) = image_id {
                    let image = renderer.image_mut(id);
                    if let Some(image) = image {
                        if skeleton.apply_bone_matrices_data(image).is_ok() {
                            skeleton.dirty = false;
                        }
                        material.values.insert(
                            "boneMatrices".to_owned(),
                            MaterialValue::Sampler2d {
                                reference: ImageReference::Id(id),
                                filtering: ImageFiltering::Nearest,
                            },
                        );
                    }
                }
            }
        }
    }
}
