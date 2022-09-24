use crate::{constants::material_uniforms::*, resources::renderables::*};
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaRenderPrototypeStageSystemCache {
    fonts_map: HashMap<String, AssetId>,
    fonts_table: HashMap<AssetId, String>,
    text_pool: Vec<(MeshId, ImageId, bool)>,
}

pub type HaRenderPrototypeStageSystemResources<'a> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a AssetsDatabase,
    &'a AppLifeCycle,
    &'a mut Renderables,
    &'a MaterialResourceMapping,
    &'a ImageResourceMapping,
    &'a MeshResourceMapping,
    &'a mut HaRenderPrototypeStageSystemCache,
    Comp<&'a mut HaCamera>,
    Comp<&'a HaVisibility>,
    Comp<&'a HaTransform>,
);

pub struct RenderPrototypeStage;

pub fn ha_render_prototype_stage_system(universe: &mut Universe) {
    let (
        world,
        mut renderer,
        assets,
        lifecycle,
        mut renderables,
        material_mapping,
        image_mapping,
        mesh_mapping,
        mut cache,
        ..,
    ) = universe.query_resources::<HaRenderPrototypeStageSystemResources>();

    for id in assets.lately_loaded_protocol("font") {
        if let Some(asset) = assets.asset_by_id(*id) {
            if asset.is::<FontAsset>() {
                cache.fonts_map.insert(asset.path().to_owned(), *id);
                cache.fonts_table.insert(*id, asset.path().to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("font") {
        if let Some(name) = cache.fonts_table.remove(id) {
            cache.fonts_map.remove(&name);
        }
    }

    renderables.collapse();
    let mut buffer = match renderables.buffer_stack.pop() {
        Some(buffer) => buffer,
        None => return,
    };
    if buffer.is_empty() {
        renderables.buffer_stack.push(buffer);
        return;
    }

    if let MeshReference::Asset(path) = &renderables.sprite_mesh_reference {
        if let Some(id) = mesh_mapping.resource_by_name(path) {
            renderables.sprite_mesh_reference = MeshReference::Id(id);
        }
    }
    if let MaterialReference::Asset(path) = &renderables.sprite_material_reference {
        if let Some(id) = material_mapping.resource_by_name(path) {
            renderables.sprite_material_reference = MaterialReference::Id(id);
        }
    }
    if let MaterialReference::Asset(path) = &renderables.text_material_reference {
        if let Some(id) = material_mapping.resource_by_name(path) {
            renderables.text_material_reference = MaterialReference::Id(id);
        }
    }

    for renderable in &mut buffer {
        if let Renderable::Advanced(renderable) = renderable {
            renderable.mesh.update_references(&mesh_mapping);
            renderable
                .material
                .update_references(&material_mapping, &image_mapping);
        }
    }

    let time = vec4(
        lifecycle.time_seconds(),
        lifecycle.delta_time_seconds(),
        lifecycle.time_seconds().fract(),
        0.0,
    );
    let mut transform_stack = Vec::<Mat4>::with_capacity(32);
    let mut blending_stack = Vec::<MaterialBlending>::with_capacity(32);
    let mut scissor_stack = Vec::<(usize, usize, usize, usize)>::with_capacity(32);

    cache.text_pool.retain_mut(|(mesh_id, _, keep)| {
        if *keep {
            *keep = false;
            true
        } else {
            let _ = renderer.remove_mesh(*mesh_id);
            false
        }
    });

    let mut text_mesh_pool_index = 0;
    for renderable in &buffer {
        if let Renderable::Text(renderable) = renderable {
            if let Some(font_asset) = cache.fonts_map.get(&renderable.font) {
                if let Some(font_asset) = assets.asset_by_id(*font_asset) {
                    if let Some(font_asset) = font_asset.get::<FontAsset>() {
                        if let Some((_, image_asset)) = font_asset.pages_image_assets.get(0) {
                            if let Some(image_id) = image_mapping.resource_by_asset(*image_asset) {
                                let text = renderable.to_text_instance();
                                if let Ok(factory) = SurfaceTextFactory::factory::<SurfaceVertexText>(
                                    &text, font_asset,
                                ) {
                                    if let Some((mesh, image, keep)) =
                                        cache.text_pool.get_mut(text_mesh_pool_index)
                                    {
                                        if let Some(m) = renderer.mesh_mut(*mesh) {
                                            m.set_vertex_storage_all(BufferStorage::Stream);
                                            m.set_index_storage(BufferStorage::Stream);
                                            if factory.write_into(m).is_ok() {
                                                *image = image_id;
                                                *keep = true;
                                                text_mesh_pool_index += 1;
                                            }
                                        }
                                    } else {
                                        if cache.text_pool.len() == cache.text_pool.capacity() {
                                            cache
                                                .text_pool
                                                .reserve(renderables.text_pool_resize_count);
                                        }
                                        let mut m = Mesh::new(factory.layout().to_owned());
                                        if factory.write_into(&mut m).is_ok() {
                                            if let Ok(mesh_id) = renderer.add_mesh(m) {
                                                cache.text_pool.push((mesh_id, image_id, true));
                                                text_mesh_pool_index += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (_, (visibility, camera, transform)) in world
        .query::<(Option<&HaVisibility>, &HaCamera, &HaTransform)>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let iter =
            match camera.record_to_pipeline_stage::<RenderPrototypeStage>(&renderer, transform) {
                Some(iter) => iter,
                None => continue,
            };

        for (info, render_queue) in iter {
            let mut render_queue = match render_queue.write() {
                Ok(render_queue) => render_queue,
                Err(_) => continue,
            };
            render_queue.clear();
            let mut recorder = render_queue.auto_recorder(None);
            transform_stack.clear();

            text_mesh_pool_index = 0;
            for renderable in &mut buffer {
                match renderable {
                    Renderable::PushTransform(transform) => {
                        let parent = transform_stack.last().copied().unwrap_or_default();
                        let current = HaTransform::from(*transform).local_matrix();
                        transform_stack.push(parent * current);
                    }
                    Renderable::PopTransform => {
                        transform_stack.pop();
                    }
                    Renderable::PushBlending(blending) => {
                        blending_stack.push(*blending);
                    }
                    Renderable::PopBlending => {
                        blending_stack.pop();
                    }
                    Renderable::PushScissor(mut scissor) => {
                        scissor = scissor.intersection(rect(0.0, 0.0, 1.0, 1.0));
                        let mut x = (scissor.x * info.width as Scalar) as usize;
                        let mut y =
                            ((1.0 - scissor.y - scissor.h) * info.height as Scalar) as usize;
                        let mut w = (scissor.w * info.width as Scalar) as usize;
                        let mut h = (scissor.h * info.height as Scalar) as usize;
                        if let Some(parent) = scissor_stack.last() {
                            let r = (x + w).min(parent.0 + parent.2);
                            let b = (y + h).min(parent.1 + parent.3);
                            x = x.max(parent.0);
                            y = y.max(parent.1);
                            w = r - x;
                            h = b - y;
                        }
                        scissor_stack.push((x, y, w, h));
                    }
                    Renderable::PopScissor => {
                        scissor_stack.pop();
                    }
                    Renderable::Advanced(renderable) => {
                        renderable.mesh.update_references(&mesh_mapping);
                        renderable
                            .material
                            .update_references(&material_mapping, &image_mapping);
                        record_commands(
                            renderable,
                            &transform_stack,
                            &scissor_stack,
                            &renderer,
                            time,
                            &info,
                            &mut recorder,
                        );
                    }
                    Renderable::Sprite(renderable) => {
                        let mut image_value = MaterialValue::sampler_2d_filter(
                            renderable.image.to_owned(),
                            renderables.sprite_filtering,
                        );
                        image_value.update_references(&image_mapping);
                        let mut material = HaMaterialInstance::new(
                            renderables.sprite_material_reference.to_owned(),
                        );
                        let mut rect = rect(0.0, 0.0, 1.0, 1.0);
                        if let MaterialValue::Sampler2d {
                            reference: ImageReference::VirtualId { owner, id },
                            ..
                        } = &image_value
                        {
                            if let Some(virtual_image) = renderer.virtual_images.get(*owner) {
                                if let Some((r, _)) = virtual_image.image_uvs(*id) {
                                    rect = r;
                                }
                            }
                        }
                        if let Some(region) = renderable.region {
                            rect = Rect {
                                x: rect.x + region.x / rect.w,
                                y: rect.y + region.y / rect.h,
                                w: region.w / rect.w,
                                h: region.h / rect.h,
                            };
                        }
                        material.values.insert(
                            MAIN_IMAGE_OFFSET_NAME.to_owned(),
                            vec2(rect.x, rect.y).into(),
                        );
                        material
                            .values
                            .insert(MAIN_IMAGE_SIZE_NAME.to_owned(), vec2(rect.w, rect.h).into());
                        material
                            .values
                            .insert(MAIN_IMAGE_TILING_NAME.to_owned(), renderable.tiling.into());
                        material
                            .values
                            .insert(MAIN_IMAGE_NAME.to_owned(), image_value);
                        material
                            .values
                            .insert(TINT_NAME.to_owned(), Vec4::from(renderable.tint).into());
                        material.override_draw_options = Some(MaterialDrawOptions {
                            blending: blending_stack
                                .last()
                                .copied()
                                .unwrap_or(MaterialBlending::Alpha),
                            ..Default::default()
                        });
                        let renderable = AdvancedRenderable {
                            transform: renderable.transform,
                            mesh: HaMeshInstance {
                                reference: renderables.sprite_mesh_reference.to_owned(),
                                ..Default::default()
                            },
                            material,
                        };
                        record_commands(
                            &renderable,
                            &transform_stack,
                            &scissor_stack,
                            &renderer,
                            time,
                            &info,
                            &mut recorder,
                        );
                    }
                    Renderable::Text(renderable) => {
                        let (mesh_id, image_id) = match cache.text_pool.get(text_mesh_pool_index) {
                            Some((mesh_id, image_id, _)) => (*mesh_id, *image_id),
                            None => continue,
                        };
                        text_mesh_pool_index += 1;
                        let mut material =
                            HaMaterialInstance::new(renderables.text_material_reference.to_owned());
                        material.values.insert(
                            MAIN_IMAGE_NAME.to_owned(),
                            MaterialValue::Sampler2dArray {
                                reference: ImageReference::Id(image_id),
                                filtering: Default::default(),
                            },
                        );
                        material.override_draw_options = Some(MaterialDrawOptions {
                            blending: blending_stack
                                .last()
                                .copied()
                                .unwrap_or(MaterialBlending::Alpha),
                            ..Default::default()
                        });
                        let renderable = AdvancedRenderable {
                            transform: renderable.transform,
                            mesh: HaMeshInstance {
                                reference: MeshReference::Id(mesh_id),
                                ..Default::default()
                            },
                            material,
                        };
                        record_commands(
                            &renderable,
                            &transform_stack,
                            &scissor_stack,
                            &renderer,
                            time,
                            &info,
                            &mut recorder,
                        );
                    }
                }
            }

            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }

    buffer.clear();
    renderables.buffer_stack.push(buffer);
}

fn record_commands(
    renderable: &AdvancedRenderable,
    transform_stack: &[Mat4],
    scissor_stack: &[(usize, usize, usize, usize)],
    renderer: &HaRenderer,
    time: Vec4,
    info: &StageProcessInfo,
    recorder: &mut RenderQueueAutoRecorder,
) {
    recorder.next_group();
    let mesh_id = match renderable.mesh.reference.id() {
        Some(id) => *id,
        None => return,
    };
    let material_id = match renderable.material.reference.id() {
        Some(id) => *id,
        None => return,
    };
    let current_mesh = match renderer.mesh(mesh_id) {
        Some(mesh) => mesh,
        None => return,
    };
    let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
    let signature = info.make_material_signature(current_mesh.layout());
    let _ = recorder.record(RenderCommand::ActivateMaterial(material_id, signature));
    let parent = transform_stack.last().copied().unwrap_or_default();
    let current = HaTransform::from(renderable.transform).local_matrix();
    let matrix = parent * current;
    let _ = recorder.record(RenderCommand::OverrideUniform(
        MODEL_MATRIX_NAME.into(),
        matrix.into(),
    ));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        VIEW_MATRIX_NAME.into(),
        info.view_matrix.into(),
    ));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        PROJECTION_MATRIX_NAME.into(),
        info.projection_matrix.into(),
    ));
    let _ = recorder.record(RenderCommand::OverrideUniform(
        TIME_NAME.into(),
        time.into(),
    ));
    for (key, value) in &renderable.material.values {
        let _ = recorder.record(RenderCommand::OverrideUniform(
            key.to_owned().into(),
            value.to_owned(),
        ));
    }
    if let Some(draw_options) = &renderable.material.override_draw_options {
        let _ = recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
    }
    let _ = recorder.record(RenderCommand::Scissor(scissor_stack.last().copied()));
    let draw_range = renderable
        .mesh
        .override_draw_range
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let _ = recorder.record(RenderCommand::DrawMesh(draw_range));
    let _ = recorder.record(RenderCommand::ResetUniforms);
}
