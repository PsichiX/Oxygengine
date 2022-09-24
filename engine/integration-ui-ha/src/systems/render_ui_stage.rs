use crate::{components::*, raui_renderer::*};
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_user_interface::{
    prelude::*,
    raui::core::{
        layout::CoordsMapping,
        widget::utils::{Rect as RauiRect, Vec2 as RauiVec2},
    },
};
use std::collections::HashMap;

const MODEL_MATRIX_NAME: &str = "model";
const VIEW_MATRIX_NAME: &str = "view";
const PROJECTION_MATRIX_NAME: &str = "projection";
const MAIN_IMAGE_NAME: &str = "mainImage";

#[derive(Debug, Default)]
pub struct HaRenderUiStageSystemCache {
    pub(crate) images_map: HashMap<String, AssetId>,
    images_table: HashMap<AssetId, String>,
    pub(crate) fonts_map: HashMap<String, AssetId>,
    fonts_table: HashMap<AssetId, String>,
    pub(crate) atlas_mapping: HashMap<String, (String, RauiRect)>,
    pub(crate) image_sizes: HashMap<String, RauiVec2>,
    meshes: HashMap<Entity, (MeshId, Vec<RenderBatch>)>,
    view_sizes: HashMap<Entity, (usize, usize)>,
    dirty: bool,
}

pub type HaRenderUiStageSystemResources<'a> = (
    WorldRef,
    &'a EntityChanges,
    &'a AssetsDatabase,
    &'a mut HaRenderer,
    &'a mut UserInterface,
    &'a ImageResourceMapping,
    &'a mut HaRenderUiStageSystemCache,
    Comp<&'a HaVisibility>,
    Comp<&'a UserInterfaceView>,
    Comp<&'a HaCamera>,
    Comp<&'a HaTransform>,
    Comp<&'a HaUserInterfaceSync>,
);

pub struct RenderUiStage;

pub fn ha_render_ui_stage_system(universe: &mut Universe) {
    let (world, changes, assets, mut renderer, mut ui, image_mapping, mut cache, ..) =
        universe.query_resources::<HaRenderUiStageSystemResources>();

    sync_cache(&renderer, &assets, &image_mapping, &mut cache);
    render(
        &world,
        &changes,
        &assets,
        &mut renderer,
        &image_mapping,
        &mut ui,
        &mut cache,
    );
}

fn sync_cache(
    renderer: &HaRenderer,
    assets: &AssetsDatabase,
    image_mapping: &ImageResourceMapping,
    cache: &mut HaRenderUiStageSystemCache,
) {
    for id in assets.lately_loaded_protocol("image") {
        if let Some(asset) = assets.asset_by_id(*id) {
            if asset.is::<ImageAsset>() {
                cache.images_map.insert(asset.path().to_owned(), *id);
                cache.images_table.insert(*id, asset.path().to_owned());
                cache.dirty = true;
            }
        }
    }
    for id in assets.lately_unloaded_protocol("image") {
        if let Some(name) = cache.images_table.remove(id) {
            cache.images_map.remove(&name);
            cache.dirty = true;
        }
    }
    for id in assets.lately_loaded_protocol("font") {
        if let Some(asset) = assets.asset_by_id(*id) {
            if asset.is::<FontAsset>() {
                cache.fonts_map.insert(asset.path().to_owned(), *id);
                cache.fonts_table.insert(*id, asset.path().to_owned());
                cache.dirty = true;
            }
        }
    }
    for id in assets.lately_unloaded_protocol("font") {
        if let Some(name) = cache.fonts_table.remove(id) {
            cache.fonts_map.remove(&name);
            cache.dirty = true;
        }
    }

    if image_mapping.removed().next().is_some()
        || image_mapping.virtual_resources_added().next().is_some()
        || image_mapping.resources_added().next().is_some()
    {
        cache.dirty = true;
    }
    for name in image_mapping.removed() {
        cache.atlas_mapping.remove(name);
        cache.image_sizes.remove(name);
    }
    cache
        .atlas_mapping
        .extend(
            image_mapping
                .virtual_resources_added()
                .filter_map(|(subname, vid, id)| {
                    let virtual_image = renderer.virtual_images.get(vid)?;
                    let image_id = virtual_image.source().image()?;
                    let name = image_mapping.resources().find(|(_, id)| image_id == *id)?.0;
                    let (uvs, _) = virtual_image.image_uvs(id)?;
                    let uvs = RauiRect {
                        left: uvs.x,
                        right: uvs.x + uvs.w,
                        top: uvs.y,
                        bottom: uvs.y + uvs.h,
                    };
                    Some((subname.to_owned(), (name.to_owned(), uvs)))
                }),
        );
    cache
        .image_sizes
        .extend(image_mapping.resources_added().filter_map(|(name, id)| {
            renderer.image(id).map(|image| {
                (
                    name.to_owned(),
                    RauiVec2 {
                        x: image.width() as _,
                        y: image.height() as _,
                    },
                )
            })
        }));
}

fn render(
    world: &World,
    changes: &EntityChanges,
    assets: &AssetsDatabase,
    renderer: &mut HaRenderer,
    image_mapping: &ImageResourceMapping,
    ui: &mut UserInterface,
    cache: &mut HaRenderUiStageSystemCache,
) {
    type V = SurfaceVertexText;

    for entity in changes.despawned() {
        if let Some((id, _)) = cache.meshes.remove(&entity) {
            let _ = renderer.remove_mesh(id);
        }
    }

    let layout = match V::vertex_layout() {
        Ok(layout) => layout,
        Err(_) => return,
    };

    for (entity, (visibility, view, camera, transform, sync)) in world
        .query::<(
            Option<&HaVisibility>,
            &UserInterfaceView,
            &HaCamera,
            &HaTransform,
            &HaUserInterfaceSync,
        )>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let (info, render_queue) =
            match camera.record_to_pipeline_stage::<RenderUiStage>(renderer, transform) {
                Some(mut iter) => match iter.next() {
                    Some(result) => result,
                    None => continue,
                },
                None => continue,
            };
        if let Some((w, h)) = cache.view_sizes.get_mut(&entity) {
            if info.width != *w || info.height != *h {
                *w = info.width;
                *h = info.height;
                cache.dirty = true;
            }
        } else {
            cache.dirty = true;
        }
        let mut ui = match ui.get_mut(view.app_id()) {
            Some(ui) => ui,
            None => continue,
        };
        let view_rect = RauiRect {
            left: 0.0,
            right: info.width as _,
            top: 0.0,
            bottom: info.height as _,
        };
        ui.coords_mapping = CoordsMapping::new_scaling(view_rect, sync.coords_mapping_scaling);
        let (mesh_id, mut batches) = match cache.meshes.get_mut(&entity) {
            Some((mesh_id, batches)) => (*mesh_id, std::mem::take(batches)),
            None => {
                let mut m = Mesh::new(layout.to_owned());
                m.set_regenerate_bounds(false);
                m.set_vertex_storage_all(BufferStorage::Dynamic);
                m.set_index_storage(BufferStorage::Dynamic);
                match renderer.add_mesh(m) {
                    Ok(mesh_id) => {
                        cache.meshes.insert(entity, (mesh_id, vec![]));
                        (mesh_id, vec![])
                    }
                    Err(_) => continue,
                }
            }
        };
        if cache.dirty || ui.application.does_render_changed() {
            let mut raui_renderer = RauiRenderer::new(cache, assets, &mut batches);
            match ui
                .application
                .render(&ui.coords_mapping, &mut raui_renderer)
            {
                Ok(factory) => match renderer.mesh_mut(mesh_id) {
                    Some(mesh) => match factory.consume_write_into(mesh) {
                        Ok(_) => cache.dirty = false,
                        Err(_) => continue,
                    },
                    None => continue,
                },
                Err(_) => continue,
            }
        }
        if batches.is_empty() {
            continue;
        }

        let mut render_queue = match render_queue.write() {
            Ok(render_queue) => render_queue,
            Err(_) => continue,
        };
        render_queue.clear();
        let colored_material_id = match sync.colored_material.reference.id() {
            Some(id) => *id,
            None => continue,
        };
        let image_material_id = match sync.image_material.reference.id() {
            Some(id) => *id,
            None => continue,
        };
        let text_material_id = match sync.text_material.reference.id() {
            Some(id) => *id,
            None => continue,
        };
        let projection_matrix = HaCameraOrthographic {
            scaling: HaCameraOrtographicScaling::None,
            centered: false,
            ignore_depth_planes: false,
        }
        .matrix(Vec2::new(info.width as _, info.height as _));
        let mut recorder = render_queue.auto_recorder(None);
        let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
        let signature = info.make_material_signature(&layout);
        let mut current_mode = DrawMode::None;
        for batch in &batches {
            match batch {
                RenderBatch::Colored(range) => {
                    let mode = DrawMode::Colored;
                    if !mode.similar(&current_mode) {
                        apply_material(
                            colored_material_id,
                            &sync.colored_material,
                            &signature,
                            Mat4::identity(),
                            projection_matrix,
                            &mut recorder,
                        );
                    }
                    current_mode = mode;
                    let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::Range(
                        range.to_owned(),
                    )));
                }
                RenderBatch::Image(id, range) => {
                    let mode = DrawMode::Image(*id);
                    if !mode.similar(&current_mode) {
                        apply_material(
                            image_material_id,
                            &sync.image_material,
                            &signature,
                            Mat4::identity(),
                            projection_matrix,
                            &mut recorder,
                        );
                    }
                    if mode != current_mode {
                        let image_id = match image_mapping.resource_by_asset(*id) {
                            Some(image_id) => image_id,
                            None => continue,
                        };
                        let _ = recorder.record(RenderCommand::OverrideUniform(
                            MAIN_IMAGE_NAME.into(),
                            MaterialValue::Sampler2d {
                                reference: ImageReference::Id(image_id),
                                filtering: sync.text_filtering,
                            },
                        ));
                    }
                    current_mode = mode;
                    let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::Range(
                        range.to_owned(),
                    )));
                }
                RenderBatch::Text(id, transform_matrix, range) => {
                    let mode = DrawMode::Text(*id);
                    if !mode.similar(&current_mode) {
                        apply_material(
                            text_material_id,
                            &sync.text_material,
                            &signature,
                            *transform_matrix,
                            projection_matrix,
                            &mut recorder,
                        );
                    }
                    if mode != current_mode {
                        let image_id = match image_mapping.resource_by_asset(*id) {
                            Some(image_id) => image_id,
                            None => continue,
                        };
                        let _ = recorder.record(RenderCommand::OverrideUniform(
                            MAIN_IMAGE_NAME.into(),
                            MaterialValue::Sampler2dArray {
                                reference: ImageReference::Id(image_id),
                                filtering: sync.text_filtering,
                            },
                        ));
                    }
                    current_mode = mode;
                    let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::Range(
                        range.to_owned(),
                    )));
                }
            }
        }

        let _ = recorder.record(RenderCommand::SortingBarrier);
        cache.meshes.get_mut(&entity).unwrap().1 = batches;
    }
}

fn apply_material(
    id: MaterialId,
    instance: &HaMaterialInstance,
    signature: &MaterialSignature,
    transform_matrix: Mat4,
    projection_matrix: Mat4,
    recorder: &mut RenderQueueAutoRecorder,
) {
    let _ = recorder.record(RenderCommand::ActivateMaterial(id, signature.to_owned()));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        MODEL_MATRIX_NAME.into(),
        transform_matrix.into(),
    ));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        VIEW_MATRIX_NAME.into(),
        Mat4::identity().into(),
    ));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        PROJECTION_MATRIX_NAME.into(),
        projection_matrix.into(),
    ));
    for (key, value) in &instance.values {
        let _ = recorder.record(RenderCommand::OverrideUniform(
            key.to_owned().into(),
            value.to_owned(),
        ));
    }
    if let Some(draw_options) = &instance.override_draw_options {
        let _ = recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DrawMode {
    None,
    Colored,
    Image(AssetId),
    Text(AssetId),
}

impl DrawMode {
    fn similar(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::None, Self::None)
                | (Self::Colored, Self::Colored)
                | (Self::Image(_), Self::Image(_))
                | (Self::Text(_), Self::Text(_))
        )
    }
}
