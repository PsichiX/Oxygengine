use crate::{
    components::{
        camera::HaCamera, material_instance::HaMaterialInstance, mesh_instance::HaMeshInstance,
        transform::HaTransform, visibility::HaVisibility,
    },
    ha_renderer::HaRenderer,
    mesh::MeshDrawRange,
    pipeline::render_queue::RenderCommand,
};
use core::ecs::{components::Tag, Comp, Universe, WorldRef};

const MODEL_MATRIX_NAME: &str = "model";
const VIEW_MATRIX_NAME: &str = "view";
const PROJECTION_MATRIX_NAME: &str = "projection";

pub type HaRenderForwardStageSystemResources<'a> = (
    WorldRef,
    &'a HaRenderer,
    Comp<&'a mut HaCamera>,
    Comp<&'a Tag>,
    Comp<&'a HaVisibility>,
    Comp<&'a HaTransform>,
    Comp<&'a HaMeshInstance>,
    Comp<&'a HaMaterialInstance>,
);

pub struct RenderForwardStage;

pub fn ha_render_forward_stage_system(universe: &mut Universe) {
    let (world, renderer, ..) = universe.query_resources::<HaRenderForwardStageSystemResources>();

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
                let _ = recorder.record(RenderCommand::ActivateMaterial {
                    id: material_id,
                    signature: signature.to_owned(),
                });
                let _ = recorder.record(RenderCommand::SubmitUniform {
                    signature: signature.to_owned(),
                    name: MODEL_MATRIX_NAME.into(),
                    value: transform.world_matrix().into(),
                });
                let _ = recorder.record(RenderCommand::SubmitUniform {
                    signature: signature.to_owned(),
                    name: VIEW_MATRIX_NAME.into(),
                    value: info.view_matrix.into(),
                });
                let _ = recorder.record(RenderCommand::SubmitUniform {
                    signature: signature.to_owned(),
                    name: PROJECTION_MATRIX_NAME.into(),
                    value: info.projection_matrix.into(),
                });
                if let Some(draw_options) = &material.override_draw_options {
                    let _ =
                        recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
                }
                for (name, value) in &material.values {
                    let _ = recorder.record(RenderCommand::SubmitUniform {
                        signature: signature.to_owned(),
                        name: name.to_owned().into(),
                        value: value.to_owned(),
                    });
                }
                // TODO: add support for virtual mesh ranges.
                let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::All));
            }

            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }
}
