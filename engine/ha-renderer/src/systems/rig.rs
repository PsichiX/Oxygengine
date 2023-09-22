use crate::{
    asset_protocols::rig::*,
    components::{material_instance::*, rig_instance::*, transform::*},
    ha_renderer::HaRenderer,
    image::*,
    material::common::MaterialValue,
    mesh::rig::*,
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
    scripting::{
        intuicio::{
            core::{context::Context, object::Object},
            data::{lifetime::*, managed::*},
        },
        Scripting,
    },
};
use std::collections::HashMap;

const CONTEXT_CAPACITY: usize = 10240;

pub struct HaRigSystemCache {
    map: HashMap<String, (Rig, AssetId)>,
    table: HashMap<AssetId, String>,
    skinning_data_images: HashMap<Entity, ImageId>,
    deformer_data_images: HashMap<Entity, ImageId>,
    control_context: Context,
}

impl Default for HaRigSystemCache {
    fn default() -> Self {
        Self {
            map: Default::default(),
            table: Default::default(),
            skinning_data_images: Default::default(),
            deformer_data_images: Default::default(),
            control_context: Context::new(CONTEXT_CAPACITY, CONTEXT_CAPACITY, CONTEXT_CAPACITY),
        }
    }
}

pub type HaRigSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a EntityChanges,
    &'a Scripting,
    &'a mut HaRenderer,
    &'a mut HaRigSystemCache,
    Comp<&'a mut HaRigInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_rig_system(universe: &mut Universe) {
    let (world, lifecycle, assets, changes, scripting, mut renderer, mut cache, ..) =
        universe.query_resources::<HaRigSystemResources>();
    let cache = &mut *cache;

    for id in assets.lately_loaded_protocol("rig") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(rig) = asset
                .get::<RigAsset>()
                .and_then(|asset| asset.build_rig(&scripting.registry).ok())
            {
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

    for (entity, (rig, material, transform)) in world
        .query::<(&mut HaRigInstance, &mut HaMaterialInstance, &HaTransform)>()
        .iter()
    {
        let dirty_deformer = rig.deformer.is_dirty();
        let dirty_skeleton = rig.skeleton.is_dirty();

        if let Some((asset, _)) = cache.map.get(rig.asset()) {
            rig.try_initialize(asset);

            if dirty_deformer {
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

            let delta_time = lifecycle.delta_time_seconds();
            rig.control.signals.clear();
            if rig.control.states.len() != asset.controls.len() {
                rig.control.states = asset
                    .controls
                    .iter()
                    .map(|control| {
                        let mut object = Object::new(control.struct_type.clone());
                        for (name, value) in &control.bindings {
                            if let Some(field) = object.write_field(name) {
                                *field = value.to_owned();
                            }
                        }
                        let (handle, memory) = unsafe { object.into_inner() };
                        DynamicManaged::new_raw(handle.type_hash(), Lifetime::default(), memory)
                    })
                    .collect();
            }
            let mut rig_states = std::mem::take(&mut rig.control.states);
            'controls: for (state, control) in rig_states.iter_mut().zip(asset.controls.iter()) {
                let token = cache.control_context.stack().store();
                let rig_lifetime = Lifetime::default();
                let asset_lifetime = Lifetime::default();
                let material_lifetime = Lifetime::default();
                let assets_lifetime = Lifetime::default();
                let transform_lifetime = Lifetime::default();
                for arg in control.solve_function.signature().inputs.iter().rev() {
                    match arg.name.as_str() {
                        "this" => {
                            let state = state.borrow_mut().unwrap();
                            cache.control_context.stack().push(state);
                        }
                        "rig" => {
                            let rig =
                                DynamicManagedRefMut::new(rig, rig_lifetime.borrow_mut().unwrap());
                            cache.control_context.stack().push(rig);
                        }
                        "asset" => {
                            let asset =
                                DynamicManagedRef::new(asset, asset_lifetime.borrow().unwrap());
                            cache.control_context.stack().push(asset);
                        }
                        "material" => {
                            let material = DynamicManagedRefMut::new(
                                material,
                                material_lifetime.borrow_mut().unwrap(),
                            );
                            cache.control_context.stack().push(material);
                        }
                        "assets" => {
                            let assets =
                                DynamicManagedRef::new(&*assets, assets_lifetime.borrow().unwrap());
                            cache.control_context.stack().push(assets);
                        }
                        "transform" => {
                            let transform = DynamicManagedRef::new(
                                transform,
                                transform_lifetime.borrow().unwrap(),
                            );
                            cache.control_context.stack().push(transform);
                        }
                        "delta_time" => {
                            cache
                                .control_context
                                .stack()
                                .push(DynamicManaged::new(delta_time));
                        }
                        _ => {
                            cache.control_context.stack().restore(token);
                            continue 'controls;
                        }
                    }
                }
                control
                    .solve_function
                    .invoke(&mut cache.control_context, &scripting.registry);
            }
            rig.control.states = rig_states;
        }
    }
}
