use crate::{
    asset_protocols::{image::ImageAsset, material::MaterialAsset, mesh::MeshAsset},
    components::{
        camera::HaCamera, material_instance::HaMaterialInstance, mesh_instance::HaMeshInstance,
        transform::HaTransform,
    },
    ha_renderer::{HaRenderer, RenderStats},
    image::{Image, ImageResourceMapping},
    material::{Material, MaterialResourceMapping},
    mesh::{Mesh, MeshResourceMapping},
    pipeline::{stage::StageQueueSorting, PipelineId},
    resources::material_library::MaterialLibrary,
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, World, WorldRef},
};
use glow::*;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, Default)]
pub struct HaRendererMaintenanceSystemCache {
    material_function_map: HashMap<AssetId, String>,
    material_domain_map: HashMap<AssetId, String>,
    pipeline_map: HashMap<Entity, PipelineId>,
}

pub type HaRendererMaintenanceSystemResources<'a> = (
    WorldRef,
    &'a EntityChanges,
    &'a mut HaRenderer,
    &'a AssetsDatabase,
    &'a mut MaterialLibrary,
    &'a mut HaRendererMaintenanceSystemCache,
    &'a mut ImageResourceMapping,
    &'a mut MeshResourceMapping,
    &'a mut MaterialResourceMapping,
    Comp<&'a mut HaCamera>,
    Comp<&'a mut HaTransform>,
    Comp<&'a mut HaMeshInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_renderer_maintenance_system(universe: &mut Universe) {
    let (
        world,
        changes,
        mut renderer,
        assets,
        mut material_library,
        mut cache,
        mut image_mapping,
        mut mesh_mapping,
        mut material_mapping,
        ..,
    ) = universe.query_resources::<HaRendererMaintenanceSystemResources>();

    renderer.errors_cache.clear();
    renderer.maintain_platform_interface();
    image_mapping.maintain();
    mesh_mapping.maintain();
    material_mapping.maintain();
    sync_cache(
        &world,
        &changes,
        &mut renderer,
        &assets,
        &mut material_library,
        &mut cache,
        &mut image_mapping,
        &mut mesh_mapping,
        &mut material_mapping,
    );
    renderer.maintain_render_targets();
    renderer.maintain_meshes();
    renderer.maintain_materials(&material_library);
    update_resource_references(&world, &image_mapping, &mesh_mapping, &material_mapping);
}

pub type HaRendererExecutionSystemResources<'a> = &'a mut HaRenderer;

pub fn ha_renderer_execution_system(universe: &mut Universe) {
    let mut renderer = universe.query_resources::<HaRendererExecutionSystemResources>();

    execute_pipelines(&mut renderer);
}

fn update_resource_references(
    world: &World,
    image_mapping: &ImageResourceMapping,
    mesh_mapping: &MeshResourceMapping,
    material_mapping: &MaterialResourceMapping,
) {
    for (_, reference) in world.query::<&mut HaMeshInstance>().iter() {
        reference.update_references(mesh_mapping);
    }
    for (_, reference) in world.query::<&mut HaMaterialInstance>().iter() {
        reference.update_references(material_mapping, image_mapping);
    }
}

fn sync_image_assets(
    renderer: &mut HaRenderer,
    assets: &AssetsDatabase,
    image_mapping: &mut ImageResourceMapping,
) {
    for id in assets.lately_loaded_protocol("image") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<ImageAsset>() {
                let image = Image::new(
                    asset.descriptor.to_owned(),
                    asset.width,
                    asset.height,
                    asset.bytes.to_owned(),
                );
                if let Ok(image) = image {
                    if let Ok(image_id) = renderer.add_image(image) {
                        image_mapping.map_asset_resource(path, *id, image_id);
                    }
                }
            }
        }
    }
    for id in assets.lately_unloaded_protocol("image") {
        if let Some(image_id) = image_mapping.unmap_asset_resource(*id) {
            let _ = renderer.remove_image(image_id);
        }
    }
}

fn sync_mesh_assets(
    renderer: &mut HaRenderer,
    assets: &AssetsDatabase,
    mesh_mapping: &mut MeshResourceMapping,
) {
    for id in assets.lately_loaded_protocol("mesh") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<MeshAsset>() {
                if let Ok(factory) = asset.factory() {
                    let mut mesh = Mesh::new(factory.layout().to_owned());
                    if factory.write_into(&mut mesh).is_ok() {
                        if let Ok(mesh_id) = renderer.add_mesh(mesh) {
                            mesh_mapping.map_asset_resource(path, *id, mesh_id);
                        }
                    }
                }
            }
        }
    }
    for id in assets.lately_unloaded_protocol("mesh") {
        if let Some(mesh_id) = mesh_mapping.unmap_asset_resource(*id) {
            let _ = renderer.remove_mesh(mesh_id);
        }
    }
}

fn sync_material_assets(
    renderer: &mut HaRenderer,
    assets: &AssetsDatabase,
    material_library: &mut MaterialLibrary,
    cache: &mut HaRendererMaintenanceSystemCache,
    material_mapping: &mut MaterialResourceMapping,
) {
    for id in assets.lately_loaded_protocol("material") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<MaterialAsset>() {
                match asset {
                    MaterialAsset::Graph {
                        default_values,
                        draw_options,
                        content,
                    } => {
                        let mut material = Material::new_graph(content.to_owned());
                        material.default_values = default_values.to_owned();
                        material.draw_options = draw_options.to_owned();
                        if let Ok(material_id) = renderer.add_material(material) {
                            material_mapping.map_asset_resource(path, *id, material_id);
                        }
                    }
                    MaterialAsset::Domain(graph) => {
                        cache.material_domain_map.insert(*id, path.to_owned());
                        material_library.add_domain(path.to_owned(), graph.to_owned());
                    }
                    MaterialAsset::Baked {
                        default_values,
                        draw_options,
                        content,
                    } => {
                        let baked = content
                            .iter()
                            .map(|baked| (baked.signature.to_owned(), baked.baked.to_owned()))
                            .collect();
                        let mut material = Material::new_baked(baked);
                        material.default_values = default_values.to_owned();
                        material.draw_options = draw_options.to_owned();
                        if let Ok(material_id) = renderer.add_material(material) {
                            material_mapping.map_asset_resource(path, *id, material_id);
                        }
                    }
                    MaterialAsset::Function(function) => {
                        cache
                            .material_function_map
                            .insert(*id, function.name.to_owned());
                        material_library.add_function(function.to_owned());
                    }
                    MaterialAsset::None => {}
                }
            }
        }
    }
    for id in assets.lately_unloaded_protocol("material") {
        if let Some(material_id) = material_mapping.unmap_asset_resource(*id) {
            let _ = renderer.remove_material(material_id);
        }
        if let Some(material_function_id) = cache.material_function_map.remove(id) {
            material_library.remove_function(&material_function_id);
        }
        if let Some(material_domain_id) = cache.material_domain_map.remove(id) {
            material_library.remove_domain(&material_domain_id);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_cache(
    world: &World,
    changes: &EntityChanges,
    renderer: &mut HaRenderer,
    assets: &AssetsDatabase,
    material_library: &mut MaterialLibrary,
    cache: &mut HaRendererMaintenanceSystemCache,
    image_mapping: &mut ImageResourceMapping,
    mesh_mapping: &mut MeshResourceMapping,
    material_mapping: &mut MaterialResourceMapping,
) {
    sync_image_assets(renderer, assets, image_mapping);
    sync_mesh_assets(renderer, assets, mesh_mapping);
    sync_material_assets(renderer, assets, material_library, cache, material_mapping);

    for entity in changes.despawned() {
        if let Some(id) = cache.pipeline_map.remove(&entity) {
            let _ = renderer.remove_pipeline(id);
        }
    }
    for (entity, camera) in world.query::<&mut HaCamera>().iter() {
        if let Entry::Vacant(entry) = cache.pipeline_map.entry(entity) {
            if let Ok(id) = renderer.add_pipeline(camera.pipeline.to_owned()) {
                entry.insert(id);
                camera.cached_pipeline = Some(id);
            }
        }
    }
}

fn execute_pipelines(renderer: &mut HaRenderer) {
    let context = match renderer.platform_interface.context() {
        Some(context) => context,
        None => return,
    };
    unsafe { context.enable(BLEND) };
    let mut stats = RenderStats::default();
    let resources = renderer.stage_resources();
    for pipeline in renderer.pipelines.values() {
        for stage in pipeline.stages.iter() {
            if let Some((_, render_target)) = pipeline.render_targets.get(&stage.render_target) {
                if let Some(render_target) = renderer.render_targets.get(*render_target) {
                    if let Ok(mut render_queue) = stage.render_queue.write() {
                        match stage.queue_sorting {
                            StageQueueSorting::Unstable => render_queue.sort_by_group_order(false),
                            StageQueueSorting::Stable => render_queue.sort_by_group_order(true),
                            _ => {}
                        }
                        let stats = &mut stats;
                        let resources = &resources;
                        let _ = render_target.render(context, stage.clear_settings, |context| {
                            let _ = render_queue.execute(context, resources, stats);
                            unsafe {
                                context.use_program(None);
                                context.bind_vertex_array(None);
                            }
                        });
                    }
                }
            }
        }
    }
    renderer.stats_cache = stats;
}
