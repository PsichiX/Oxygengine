use crate::request::*;
use core::ecs::Universe;
use editor::simp::*;
use renderer::{
    ha_renderer::HaRenderer,
    image::*,
    material::{common::*, *},
    mesh::*,
    pipeline::{render_queue::*, *},
    render_target::*,
};
use std::collections::HashSet;

pub struct HaRendererDebuggerSystemCache<C>(C)
where
    C: SimpChannel + Sized;

impl<C> HaRendererDebuggerSystemCache<C>
where
    C: SimpChannel + Sized,
{
    pub fn new(channel: C) -> Self {
        Self(channel)
    }
}

pub type HaRendererDebuggerSystemResources<'a, C> =
    (&'a HaRenderer, &'a mut HaRendererDebuggerSystemCache<C>);

pub fn ha_renderer_debugger_system<C>(universe: &mut Universe)
where
    C: SimpChannel + 'static,
{
    let (renderer, mut cache) = universe.query_resources::<HaRendererDebuggerSystemResources<C>>();
    let renderer = &renderer;

    while let Some(message) = cache.0.read() {
        let message = match message {
            Ok(message) => message,
            Err(_) => continue,
        };
        core::info!("[Request]: {}", message.id);
        let response = match message.id.id.as_str() {
            "CheckPulse" => Response::CheckPulse,
            "TakeSnapshot" => Response::TakeSnapshot(ResponseTakeSnapshot {
                stages: match list_stages(renderer) {
                    Response::ListStages(items) => items,
                    _ => unreachable!(),
                },
                pipelines: renderer
                    .pipelines()
                    .filter_map(|id| match query_pipeline(id, renderer) {
                        Response::QueryPipeline(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                pipelines_render_queues: renderer
                    .pipelines()
                    .flat_map(|id| {
                        let count = renderer.pipeline(id).unwrap().stages_count();
                        (0..count).filter_map(move |index| match query_pipeline_stage_render_queue(
                            id, index, renderer,
                        ) {
                            Response::QueryPipelineStageRenderQueue(item) => Some(item),
                            _ => None,
                        })
                    })
                    .collect(),
                render_targets: renderer
                    .render_targets()
                    .ids()
                    .filter_map(|id| match query_render_target(id, renderer) {
                        Response::QueryRenderTarget(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                render_targets_color_data: renderer
                    .render_targets()
                    .ids()
                    .flat_map(|id| {
                        let count = renderer
                            .render_targets()
                            .get(id)
                            .unwrap()
                            .buffers()
                            .colors
                            .len();
                        (0..count).filter_map(move |index| {
                            match query_render_target_color_data(id, index, renderer) {
                                Response::QueryRenderTargetColorData(item) => Some(item),
                                _ => None,
                            }
                        })
                    })
                    .collect(),
                meshes: renderer
                    .meshes()
                    .ids()
                    .filter_map(|id| match query_mesh(id, renderer) {
                        Response::QueryMesh(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                meshes_data: renderer
                    .meshes()
                    .ids()
                    .filter_map(|id| match query_mesh_data(id, renderer) {
                        Response::QueryMeshData(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                images: renderer
                    .images()
                    .ids()
                    .filter_map(|id| match query_image(id, renderer) {
                        Response::QueryImage(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                images_data: renderer
                    .images()
                    .ids()
                    .filter_map(|id| match query_image_data(id, renderer) {
                        Response::QueryImageData(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
                materials: renderer
                    .materials()
                    .ids()
                    .filter_map(|id| match query_material(id, renderer) {
                        Response::QueryMaterial(item) => Some(item),
                        _ => None,
                    })
                    .collect(),
            }),
            "ListStages" => list_stages(renderer),
            "ListPipelines" => Response::ListPipelines(renderer.pipelines().collect()),
            "ListRenderTargets" => {
                Response::ListRenderTargets(renderer.render_targets().ids().collect())
            }
            "ListMeshes" => Response::ListMeshes(renderer.meshes().ids().collect()),
            "ListImages" => Response::ListImages(renderer.images().ids().collect()),
            "ListMaterials" => Response::ListMaterials(renderer.materials().ids().collect()),
            "QueryPipeline" => {
                let id = request_args::<PipelineId>(&message);
                query_pipeline(id, renderer)
            }
            "QueryPipelineResources" => {
                let id = request_args::<PipelineId>(&message);
                renderer
                    .pipeline(id)
                    .map(|pipeline| {
                        let mut result = ResponseQueryPipelineResources {
                            id,
                            render_targets: HashSet::default(),
                            meshes: HashSet::default(),
                            images: HashSet::default(),
                            materials: HashSet::default(),
                        };
                        result.render_targets.extend(pipeline.render_targets());
                        for stage_index in 0..pipeline.stages_count() {
                            if let Some(render_queue) =
                                pipeline.cloned_stage_render_queue(stage_index)
                            {
                                let (meshes, images, materials) =
                                    collect_render_queue_resources(&render_queue, renderer);
                                result.meshes.extend(meshes);
                                result.images.extend(images);
                                result.materials.extend(materials);
                            } else {
                                return Response::PipelineStageDoesNotExists(
                                    ResponsePipelineStageDoesNotExists { id, stage_index },
                                );
                            }
                        }
                        Response::QueryPipelineResources(result)
                    })
                    .unwrap_or_else(|| Response::PipelineDoesNotExists(id))
            }
            "QueryPipelineStageRenderQueue" => {
                let RequestQueryPipelineStageRenderQueue { id, stage_index } =
                    request_args(&message);
                query_pipeline_stage_render_queue(id, stage_index, renderer)
            }
            "QueryPipelineStageRenderQueueResources" => {
                let RequestQueryPipelineStageRenderQueueResources { id, stage_index } =
                    request_args(&message);
                renderer
                    .pipeline(id)
                    .map(|pipeline| {
                        pipeline
                            .cloned_stage_render_queue(stage_index)
                            .map(|render_queue| {
                                let (meshes, images, materials) =
                                    collect_render_queue_resources(&render_queue, renderer);
                                Response::QueryPipelineStageRenderQueueResources(
                                    ResponseQueryPipelineStageRenderQueueResources {
                                        id,
                                        stage_index,
                                        meshes,
                                        images,
                                        materials,
                                    },
                                )
                            })
                            .unwrap_or_else(|| {
                                Response::PipelineStageDoesNotExists(
                                    ResponsePipelineStageDoesNotExists { id, stage_index },
                                )
                            })
                    })
                    .unwrap_or_else(|| Response::PipelineDoesNotExists(id))
            }
            "QueryRenderTarget" => {
                let id = request_args::<RenderTargetId>(&message);
                query_render_target(id, renderer)
            }
            "QueryRenderTargetColorData" => {
                let RequestQueryRenderTargetColorData {
                    id,
                    attachment_index,
                } = request_args(&message);
                query_render_target_color_data(id, attachment_index, renderer)
            }
            "QueryMesh" => {
                let id = request_args::<MeshId>(&message);
                query_mesh(id, renderer)
            }
            "QueryMeshVertexData" => {
                let RequestQueryMeshVertexData { id, buffer_index } = request_args(&message);
                renderer
                    .mesh(id)
                    .map(|mesh| {
                        mesh.layout()
                            .buffers()
                            .get(buffer_index)
                            .map(|layout| {
                                mesh.vertex_data(buffer_index)
                                    .map(|data| {
                                        Response::QueryMeshVertexData(ResponseQueryMeshVertexData {
                                            id,
                                            buffer_index,
                                            layout: layout.to_owned(),
                                            bytes: unsafe { data.align_to().1.to_owned() },
                                        })
                                    })
                                    .unwrap_or_else(|| {
                                        Response::MeshVertexBufferDoesNotExists(
                                            ResponseMeshVertexBufferDoesNotExists {
                                                id,
                                                buffer_index,
                                            },
                                        )
                                    })
                            })
                            .unwrap_or_else(|| {
                                Response::MeshVertexBufferDoesNotExists(
                                    ResponseMeshVertexBufferDoesNotExists { id, buffer_index },
                                )
                            })
                    })
                    .unwrap_or_else(|| Response::MeshDoesNotExists(id))
            }
            "QueryMeshIndexData" => {
                let id = request_args::<MeshId>(&message);
                renderer
                    .mesh(id)
                    .map(|mesh| {
                        Response::QueryMeshIndexData(ResponseQueryMeshIndexData {
                            id,
                            draw_mode: mesh.draw_mode(),
                            bytes: unsafe { mesh.index_data().align_to().1.to_owned() },
                        })
                    })
                    .unwrap_or_else(|| Response::MeshDoesNotExists(id))
            }
            "QueryMeshData" => {
                let id = request_args::<MeshId>(&message);
                query_mesh_data(id, renderer)
            }
            "QueryImage" => {
                let id = request_args::<ImageId>(&message);
                query_image(id, renderer)
            }
            "QueryImageData" => {
                let id = request_args::<ImageId>(&message);
                query_image_data(id, renderer)
            }
            "QueryMaterial" => {
                let id = request_args::<MaterialId>(&message);
                query_material(id, renderer)
            }
            id => Response::UnknownRequest(id.to_owned()),
        };
        let message: SimpMessage = response.into();
        core::info!("[Response]: {}", message.id);
        let _ = cache.0.write(message);
    }
}

fn list_stages(renderer: &HaRenderer) -> Response {
    Response::ListStages(
        renderer
            .stages()
            .map(|(n, _, tn)| ResponseStage {
                stage_name: n.to_owned(),
                type_name: tn.to_owned(),
            })
            .collect(),
    )
}

fn query_pipeline(id: PipelineId, renderer: &HaRenderer) -> Response {
    renderer
        .pipeline(id)
        .map(|pipeline| {
            Response::QueryPipeline(ResponseQueryPipeline {
                id,
                info: pipeline.detailed_info(),
            })
        })
        .unwrap_or_else(|| Response::PipelineDoesNotExists(id))
}

fn query_pipeline_stage_render_queue(
    id: PipelineId,
    stage_index: usize,
    renderer: &HaRenderer,
) -> Response {
    renderer
        .pipeline(id)
        .map(|pipeline| {
            pipeline
                .cloned_stage_render_queue(stage_index)
                .map(|render_queue| {
                    Response::QueryPipelineStageRenderQueue(ResponseQueryPipelineStageRenderQueue {
                        id,
                        stage_index,
                        render_queue,
                    })
                })
                .unwrap_or_else(|| {
                    Response::PipelineStageDoesNotExists(ResponsePipelineStageDoesNotExists {
                        id,
                        stage_index,
                    })
                })
        })
        .unwrap_or_else(|| Response::PipelineDoesNotExists(id))
}

fn query_render_target(id: RenderTargetId, renderer: &HaRenderer) -> Response {
    renderer
        .render_target(id)
        .map(|render_target| {
            Response::QueryRenderTarget(ResponseQueryRenderTarget {
                id,
                info: render_target.detailed_info(),
            })
        })
        .unwrap_or_else(|| Response::RenderTargetDoesNotExists(id))
}

fn query_render_target_color_data(
    id: RenderTargetId,
    attachment_index: usize,
    renderer: &HaRenderer,
) -> Response {
    renderer
        .render_target(id)
        .map(|render_target| {
            renderer
                .interface()
                .context()
                .map(|context| {
                    render_target
                        .buffers()
                        .colors
                        .get(attachment_index)
                        .map(|buffer| {
                            render_target
                                .query_color_data(attachment_index, context)
                                .map(|data| {
                                    Response::QueryRenderTargetColorData(
                                        ResponseQueryRenderTargetColorData {
                                            id,
                                            attachment_index,
                                            width: render_target.width(),
                                            height: render_target.height(),
                                            value_type: buffer.value_type,
                                            bytes: data,
                                        },
                                    )
                                })
                                .unwrap_or_else(|_| Response::RenderTargetHasNoGpuResource(id))
                        })
                        .unwrap_or_else(|| {
                            Response::RenderTargetHasNoColorBuffer(
                                ResponseRenderTargetHasNoColorBuffer {
                                    id,
                                    attachment_index,
                                },
                            )
                        })
                })
                .unwrap_or_else(|| Response::RendererHasNoContext)
        })
        .unwrap_or_else(|| Response::RenderTargetDoesNotExists(id))
}

fn query_mesh(id: MeshId, renderer: &HaRenderer) -> Response {
    renderer
        .mesh(id)
        .map(|mesh| {
            Response::QueryMesh(ResponseQueryMesh {
                id,
                info: mesh.detailed_info(),
            })
        })
        .unwrap_or_else(|| Response::MeshDoesNotExists(id))
}

fn query_mesh_data(id: MeshId, renderer: &HaRenderer) -> Response {
    renderer
        .mesh(id)
        .map(|mesh| {
            Response::QueryMeshData(ResponseQueryMeshData {
                id,
                layout: mesh.layout().to_owned(),
                draw_mode: mesh.draw_mode(),
                vertex_bytes: (0..mesh.layout().buffers().len())
                    .map(|index| unsafe {
                        mesh.vertex_data(index).unwrap().align_to().1.to_owned()
                    })
                    .collect(),
                index_bytes: unsafe { mesh.index_data().align_to().1.to_owned() },
            })
        })
        .unwrap_or_else(|| Response::MeshDoesNotExists(id))
}

fn query_image(id: ImageId, renderer: &HaRenderer) -> Response {
    renderer
        .image(id)
        .map(|image| {
            Response::QueryImage(ResponseQueryImage {
                id,
                info: image.detailed_info(),
            })
        })
        .unwrap_or_else(|| Response::ImageDoesNotExists(id))
}

fn query_image_data(id: ImageId, renderer: &HaRenderer) -> Response {
    renderer
        .image(id)
        .map(|image| {
            Response::QueryImageData(ResponseQueryImageData {
                id,
                width: image.width(),
                height: image.height(),
                depth: image.depth(),
                format: image.format(),
                bytes: image.data().to_owned(),
            })
        })
        .unwrap_or_else(|| Response::ImageDoesNotExists(id))
}

fn query_material(id: MaterialId, renderer: &HaRenderer) -> Response {
    renderer
        .material(id)
        .map(|material| {
            Response::QueryMaterial(ResponseQueryMaterial {
                id,
                info: material.detailed_info().into(),
            })
        })
        .unwrap_or_else(|| Response::MaterialDoesNotExists(id))
}

fn collect_render_queue_resources(
    render_queue: &RenderQueue,
    renderer: &HaRenderer,
) -> (
    HashSet<MeshId>,
    HashSet<ImageId>,
    HashSet<(MaterialId, MaterialSignature)>,
) {
    let materials = render_queue
        .iter()
        .filter_map(|command| match command {
            RenderCommand::ActivateMaterial(id, signature) => Some((*id, signature.to_owned())),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let meshes = render_queue
        .iter()
        .filter_map(|command| match command {
            RenderCommand::ActivateMesh(id) => Some(*id),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let mut images = render_queue
        .iter()
        .filter_map(|command| match command {
            RenderCommand::OverrideUniform(_, value) => image_from_material_value(value),
            _ => None,
        })
        .collect::<HashSet<_>>();
    for material in materials
        .iter()
        .filter_map(|(id, _)| renderer.material(*id))
    {
        images.extend(
            material
                .default_values
                .values()
                .filter_map(image_from_material_value),
        );
    }
    (meshes, images, materials)
}

fn image_from_material_value(value: &MaterialValue) -> Option<ImageId> {
    let reference = match value {
        MaterialValue::Sampler2d { reference, .. } => Some(reference),
        MaterialValue::Sampler2dArray { reference, .. } => Some(reference),
        MaterialValue::Sampler3d { reference, .. } => Some(reference),
        _ => None,
    };
    reference.and_then(|reference| match reference {
        ImageReference::Id(id) => Some(*id),
        ImageReference::VirtualId { id, .. } => Some(*id),
        _ => None,
    })
}
