use crate::{
    asset_protocols::rig::*,
    components::{material_instance::*, rig_instance::*},
    ha_renderer::HaRenderer,
    image::*,
    material::common::MaterialValue,
    mesh::rig::*,
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaRigSystemCache {
    map: HashMap<String, (Rig, AssetId)>,
    table: HashMap<AssetId, String>,
    skinning_data_images: HashMap<Entity, ImageId>,
    deformer_data_images: HashMap<Entity, ImageId>,
}

pub type HaRigSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a EntityChanges,
    &'a mut HaRenderer,
    &'a mut HaRigSystemCache,
    Comp<&'a mut HaRigInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_rig_system(universe: &mut Universe) {
    let (world, assets, changes, mut renderer, mut cache, ..) =
        universe.query_resources::<HaRigSystemResources>();

    for id in assets.lately_loaded_protocol("rig") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(rig) = asset.get::<RigAsset>().and_then(|asset| asset.rig().ok()) {
                cache.map.insert(path.to_owned(), (rig, *id));
                cache.table.insert(*id, path.to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("rig") {
        if let Some(name) = cache.table.remove(id) {
            cache.map.remove(&name);
        }
    }
    for entity in changes.despawned() {
        if let Some(id) = cache.skinning_data_images.remove(&entity) {
            let _ = renderer.remove_image(id);
        }
        if let Some(id) = cache.deformer_data_images.remove(&entity) {
            let _ = renderer.remove_image(id);
        }
    }

    for (entity, (rig, material)) in world
        .query::<(&mut HaRigInstance, &mut HaMaterialInstance)>()
        .iter()
    {
        let dirty_deformer = rig.deformer.is_dirty();
        let dirty_skeleton = rig.skeleton.is_dirty();
        if !dirty_deformer && !dirty_skeleton {
            continue;
        }

        if let Some((asset, _)) = cache.map.get(rig.asset()) {
            rig.try_initialize(asset);

            if rig.deformer.is_dirty() {
                let image_id = cache
                    .deformer_data_images
                    .get(&entity)
                    .copied()
                    .or_else(|| {
                        let descriptor = ImageDescriptor {
                            mode: ImageMode::Image2d,
                            format: ImageFormat::Data,
                            mipmap: ImageMipmap::None,
                        };
                        let data = unsafe { [0.0_f32, 0.0, 0.0, 0.0].align_to::<u8>().1 }.to_vec();
                        let image = Image::new(descriptor, 1, 1, 1, data);
                        image
                            .ok()
                            .and_then(|image| renderer.add_image(image).ok())
                            .map(|image_id| {
                                cache.deformer_data_images.insert(entity, image_id);
                                image_id
                            })
                    });
                if let Some(id) = image_id {
                    let image = renderer.image_mut(id);
                    if let Some(image) = image {
                        if rig.deformer.apply_areas_data(image).is_ok() {
                            rig.deformer.unmark_dirty();
                        }
                        material.values.insert(
                            "bezierCurves".to_owned(),
                            MaterialValue::Sampler2d {
                                reference: ImageReference::Id(id),
                                filtering: ImageFiltering::Nearest,
                            },
                        );
                    }
                }
            }

            if dirty_skeleton {
                if let Some((asset, _)) = cache.map.get(rig.asset()) {
                    rig.skeleton.recalculate_bone_matrices(&asset.skeleton);
                    let image_id = cache
                        .skinning_data_images
                        .get(&entity)
                        .copied()
                        .or_else(|| {
                            let descriptor = ImageDescriptor {
                                mode: ImageMode::Image2d,
                                format: ImageFormat::Data,
                                mipmap: ImageMipmap::None,
                            };
                            let data =
                                unsafe { [0.0_f32, 0.0, 0.0, 0.0].align_to::<u8>().1 }.to_vec();
                            let image = Image::new(descriptor, 1, 1, 1, data);
                            image
                                .ok()
                                .and_then(|image| renderer.add_image(image).ok())
                                .map(|image_id| {
                                    cache.skinning_data_images.insert(entity, image_id);
                                    image_id
                                })
                        });
                    if let Some(id) = image_id {
                        let image = renderer.image_mut(id);
                        if let Some(image) = image {
                            if rig.skeleton.apply_bone_matrices_data(image).is_ok() {
                                rig.skeleton.unmark_dirty();
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
}