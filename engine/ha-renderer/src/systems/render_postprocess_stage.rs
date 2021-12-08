use crate::{
    components::{camera::HaCamera, postprocess::HaPostProcess, visibility::HaVisibility},
    ha_renderer::HaRenderer,
    image::ImageResourceMapping,
    material::{
        domains::screenspace::{ScreenSpaceQuadFactory, ScreenSpaceVertex},
        MaterialResourceMapping,
    },
    mesh::{vertex_factory::VertexType, Mesh, MeshDrawRange, MeshId},
    pipeline::render_queue::RenderCommand,
};
use core::ecs::{Comp, Universe, WorldRef};

#[derive(Debug, Default, Clone)]
pub struct HaRenderPostProcessStageSystemCache {
    mesh: Option<MeshId>,
}

pub type HaRenderPostProcessStageSystemResources<'a> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a MaterialResourceMapping,
    &'a ImageResourceMapping,
    &'a mut HaRenderPostProcessStageSystemCache,
    Comp<&'a mut HaCamera>,
    Comp<&'a HaVisibility>,
    Comp<&'a mut HaPostProcess>,
);

pub struct RenderPostProcessStage;

pub fn ha_render_postprocess_stage_system(universe: &mut Universe) {
    type V = ScreenSpaceVertex;

    let (world, mut renderer, material_mapping, image_mapping, mut cache, ..) =
        universe.query_resources::<HaRenderPostProcessStageSystemResources>();

    let layout = match V::vertex_layout() {
        Ok(layout) => layout,
        Err(_) => return,
    };

    let mesh_id = match cache.mesh {
        Some(mesh_id) => mesh_id,
        None => {
            let mut m = Mesh::new(layout.to_owned());
            match ScreenSpaceQuadFactory.factory() {
                Ok(factory) => {
                    if factory.write_into(&mut m).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
            match renderer.add_mesh(m) {
                Ok(mesh_id) => {
                    cache.mesh = Some(mesh_id);
                    mesh_id
                }
                Err(_) => return,
            }
        }
    };

    let transform = Default::default();
    for (_, (visibility, camera, postprocess)) in world
        .query::<(Option<&HaVisibility>, &HaCamera, &mut HaPostProcess)>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let iter = match camera
            .record_to_pipeline_stage::<RenderPostProcessStage>(&renderer, &transform)
        {
            Some(iter) => iter,
            None => continue,
        };
        for (index, (info, render_queue)) in iter.enumerate() {
            let material = match postprocess.stages.get_mut(index) {
                Some(material) => material,
                None => return,
            };
            material.update_references(&material_mapping, &image_mapping);
            let material_id = match material.reference.id().copied() {
                Some(material_id) => material_id,
                None => return,
            };
            let mut render_queue = match render_queue.write() {
                Ok(render_queue) => render_queue,
                Err(_) => continue,
            };
            render_queue.clear();
            let mut recorder = render_queue.auto_recorder(None);

            let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
            let signature = info.make_material_signature(&layout);
            let _ = recorder.record(RenderCommand::ActivateMaterial {
                id: material_id,
                signature: signature.to_owned(),
            });
            if let Some(draw_options) = &material.override_draw_options {
                let _ = recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
            }
            for (name, value) in &material.values {
                let _ = recorder.record(RenderCommand::SubmitUniform {
                    signature: signature.to_owned(),
                    name: name.to_owned().into(),
                    value: value.to_owned(),
                });
            }
            let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::All));
            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }
}
