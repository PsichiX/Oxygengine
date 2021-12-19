use crate::{
    components::{camera::HaCamera, transform::HaTransform, visibility::HaVisibility},
    ha_renderer::HaRenderer,
    image::ImageResourceMapping,
    material::{domains::gizmo::GizmoVertex, MaterialResourceMapping},
    math::*,
    mesh::{vertex_factory::VertexType, BufferStorage, Mesh, MeshDrawRange, MeshId},
    pipeline::render_queue::RenderCommand,
    resources::gizmos::Gizmos,
};
use core::ecs::{Comp, Universe, WorldRef};

const MODEL_MATRIX_NAME: &str = "model";
const VIEW_MATRIX_NAME: &str = "view";
const PROJECTION_MATRIX_NAME: &str = "projection";

#[derive(Debug, Default, Clone)]
pub struct HaRenderGizmoStageSystemCache {
    mesh: Option<MeshId>,
}

pub type HaRenderGizmoStageSystemResources<'a> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a mut Gizmos,
    &'a MaterialResourceMapping,
    &'a ImageResourceMapping,
    &'a mut HaRenderGizmoStageSystemCache,
    Comp<&'a mut HaCamera>,
    Comp<&'a HaVisibility>,
    Comp<&'a HaTransform>,
);

pub struct RenderGizmoStage;

pub fn ha_render_gizmo_stage_system(universe: &mut Universe) {
    type V = GizmoVertex;

    let (world, mut renderer, mut gizmos, material_mapping, image_mapping, mut cache, ..) =
        universe.query_resources::<HaRenderGizmoStageSystemResources>();

    if gizmos.factory.is_empty() {
        return;
    }

    let layout = match V::vertex_layout() {
        Ok(layout) => layout,
        Err(_) => return,
    };

    let mesh_id = match cache.mesh {
        Some(mesh_id) => mesh_id,
        None => {
            let mut m = Mesh::new(layout.to_owned());
            m.set_regenerate_bounds(false);
            m.set_vertex_storage_all(BufferStorage::Dynamic);
            m.set_index_storage(BufferStorage::Dynamic);
            match renderer.add_mesh(m) {
                Ok(mesh_id) => {
                    cache.mesh = Some(mesh_id);
                    mesh_id
                }
                Err(_) => return,
            }
        }
    };
    match renderer.mesh_mut(mesh_id) {
        Some(mesh) => match gizmos.factory.factory() {
            Ok(factory) => {
                if factory.write_into(mesh).is_err() {
                    return;
                }
            }
            Err(_) => return,
        },
        None => return,
    }

    gizmos
        .material
        .update_references(&material_mapping, &image_mapping);
    let material_id = match gizmos.material.reference.id().copied() {
        Some(material_id) => material_id,
        None => return,
    };

    for (_, (visibility, camera, transform)) in world
        .query::<(Option<&HaVisibility>, &HaCamera, &HaTransform)>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let iter = match camera.record_to_pipeline_stage::<RenderGizmoStage>(&renderer, transform) {
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

            let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
            let signature = info.make_material_signature(&layout);
            let _ = recorder.record(RenderCommand::ActivateMaterial(
                material_id,
                signature.to_owned(),
            ));
            let _ = recorder.record(RenderCommand::OverrideUniform(
                MODEL_MATRIX_NAME.into(),
                Mat4::identity().into(),
            ));
            let _ = recorder.record(RenderCommand::OverrideUniform(
                VIEW_MATRIX_NAME.into(),
                info.view_matrix.into(),
            ));
            let _ = recorder.record(RenderCommand::OverrideUniform(
                PROJECTION_MATRIX_NAME.into(),
                info.projection_matrix.into(),
            ));
            for (key, value) in &gizmos.material.values {
                let _ = recorder.record(RenderCommand::OverrideUniform(
                    key.to_owned().into(),
                    value.to_owned(),
                ));
            }
            if let Some(draw_options) = &gizmos.material.override_draw_options {
                let _ = recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
            }
            let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::All));
            let _ = recorder.record(RenderCommand::ResetUniforms);
            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }

    gizmos.factory.clear();
}
