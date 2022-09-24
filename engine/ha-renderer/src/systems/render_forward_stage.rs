use crate::{
    components::{
        camera::HaCamera, material_instance::HaMaterialInstance, mesh_instance::HaMeshInstance,
        transform::HaTransform, visibility::HaVisibility,
    },
    constants::material_uniforms::*,
    ha_renderer::HaRenderer,
    math::*,
    pipeline::render_queue::RenderCommand,
};
use core::{
    app::AppLifeCycle,
    ecs::{components::Tag, Comp, Universe, WorldRef},
};

pub type HaRenderForwardStageSystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    &'a AppLifeCycle,
    Comp<&'a mut HaCamera>,
    Comp<&'a Tag>,
    Comp<&'a HaVisibility>,
    Comp<&'a HaTransform>,
    Comp<&'a HaMeshInstance>,
    Comp<&'a HaMaterialInstance>,
);

pub struct RenderForwardStage;

pub fn ha_render_forward_stage_system(universe: &mut Universe) {
    let (world, renderer, lifecycle, ..) =
        universe.query_resources::<HaRenderForwardStageSystemResources>();

    let time = vec4(
        lifecycle.time_seconds(),
        lifecycle.delta_time_seconds(),
        lifecycle.time_seconds().fract(),
        0.0,
    );

    for (_, (visibility, camera, transform)) in world
        .query::<(Option<&HaVisibility>, &HaCamera, &HaTransform)>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let iter = match camera.record_to_pipeline_stage::<RenderForwardStage>(&renderer, transform)
        {
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

            for (transform, mesh, material) in world
                .query::<(
                    Option<&Tag>,
                    Option<&HaVisibility>,
                    &HaTransform,
                    &HaMeshInstance,
                    &HaMaterialInstance,
                )>()
                .iter()
                .filter(|(_, (tag, visibility, _, _, _))| {
                    visibility.map(|v| v.0).unwrap_or(true)
                        && tag.map(|t| info.filters.validate_tag(&t.0)).unwrap_or(true)
                })
                .map(|(_, (_, _, transform, mesh, material))| (transform, mesh, material))
            {
                recorder.next_group();
                let mesh_id = match mesh.reference.id() {
                    Some(id) => *id,
                    None => continue,
                };
                let material_id = match material.reference.id() {
                    Some(id) => *id,
                    None => continue,
                };
                let current_mesh = match renderer.mesh(mesh_id) {
                    Some(mesh) => mesh,
                    None => continue,
                };
                let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
                let signature = info.make_material_signature(current_mesh.layout());
                let _ = recorder.record(RenderCommand::ActivateMaterial(
                    material_id,
                    signature.to_owned(),
                ));
                let _ = recorder.record(RenderCommand::OverrideUniform(
                    MODEL_MATRIX_NAME.into(),
                    transform.world_matrix().into(),
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
                for (key, value) in &material.values {
                    let _ = recorder.record(RenderCommand::OverrideUniform(
                        key.to_owned().into(),
                        value.to_owned(),
                    ));
                }
                if let Some(draw_options) = &material.override_draw_options {
                    let _ =
                        recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
                }
                let draw_range = mesh
                    .override_draw_range
                    .as_ref()
                    .cloned()
                    .unwrap_or_default();
                let _ = recorder.record(RenderCommand::DrawMesh(draw_range));
                let _ = recorder.record(RenderCommand::ResetUniforms);
            }

            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }
}
