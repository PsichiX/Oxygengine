use editor::simp::*;
use renderer::{
    image::*,
    material::{common::*, *},
    mesh::*,
    pipeline::{render_queue::*, *},
    render_target::*,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

const VERSION: u32 = 0;

pub fn request_args<T: DeserializeOwned>(message: &SimpMessage) -> T {
    serde_json::from_str(message.text_data.as_ref().unwrap()).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseStage {
    pub stage_name: String,
    pub type_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryPipeline {
    pub id: PipelineId,
    pub info: PipelineDetailedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryPipelineResources {
    pub id: PipelineId,
    pub render_targets: HashSet<RenderTargetId>,
    pub meshes: HashSet<MeshId>,
    pub images: HashSet<ImageId>,
    pub materials: HashSet<(MaterialId, MaterialSignature)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestQueryPipelineStageRenderQueue {
    pub id: PipelineId,
    pub stage_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryPipelineStageRenderQueue {
    pub id: PipelineId,
    pub stage_index: usize,
    pub render_queue: RenderQueue,
}

pub type RequestQueryPipelineStageRenderQueueResources = RequestQueryPipelineStageRenderQueue;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryPipelineStageRenderQueueResources {
    pub id: PipelineId,
    pub stage_index: usize,
    pub meshes: HashSet<MeshId>,
    pub images: HashSet<ImageId>,
    pub materials: HashSet<(MaterialId, MaterialSignature)>,
}

pub type ResponsePipelineStageDoesNotExists = RequestQueryPipelineStageRenderQueue;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryRenderTarget {
    pub id: RenderTargetId,
    pub info: RenderTargetDetailedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestQueryRenderTargetColorData {
    pub id: RenderTargetId,
    pub attachment_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryRenderTargetColorData {
    pub id: RenderTargetId,
    pub attachment_index: usize,
    pub width: usize,
    pub height: usize,
    pub value_type: TargetValueType,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryRenderTargetColorDataText {
    pub id: RenderTargetId,
    pub attachment_index: usize,
    pub width: usize,
    pub height: usize,
    pub value_type: TargetValueType,
    pub bytes_range: Range<usize>,
}

pub type ResponseRenderTargetHasNoColorBuffer = RequestQueryRenderTargetColorData;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMesh {
    pub id: MeshId,
    pub info: MeshDetailedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestQueryMeshVertexData {
    pub id: MeshId,
    pub buffer_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshVertexData {
    pub id: MeshId,
    pub buffer_index: usize,
    pub layout: VertexBufferLayout,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshVertexDataText {
    pub id: MeshId,
    pub buffer_index: usize,
    pub layout: VertexBufferLayout,
    pub bytes_range: Range<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshIndexData {
    pub id: MeshId,
    pub draw_mode: MeshDrawMode,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshIndexDataText {
    pub id: MeshId,
    pub draw_mode: MeshDrawMode,
    pub bytes_range: Range<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshData {
    pub id: MeshId,
    pub layout: VertexLayout,
    pub draw_mode: MeshDrawMode,
    pub vertex_bytes: Vec<Vec<u8>>,
    pub index_bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMeshDataText {
    pub id: MeshId,
    pub layout: VertexLayout,
    pub draw_mode: MeshDrawMode,
    pub vertex_bytes_ranges: Vec<Range<usize>>,
    pub index_bytes_range: Range<usize>,
}

pub type ResponseMeshVertexBufferDoesNotExists = RequestQueryMeshVertexData;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryImage {
    pub id: ImageId,
    pub info: ImageDetailedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryImageData {
    pub id: ImageId,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub format: ImageFormat,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryImageDataText {
    pub id: ImageId,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub format: ImageFormat,
    pub bytes_range: Range<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMaterialDetailedInfo {
    pub versions: Vec<(MaterialSignature, BakedMaterialShaders)>,
    pub default_values: HashMap<String, MaterialValue>,
}

impl From<MaterialDetailedInfo> for ResponseQueryMaterialDetailedInfo {
    fn from(info: MaterialDetailedInfo) -> Self {
        Self {
            versions: info.versions.into_iter().collect::<Vec<_>>(),
            default_values: info.default_values,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQueryMaterial {
    pub id: MaterialId,
    pub info: ResponseQueryMaterialDetailedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTakeSnapshot {
    pub stages: Vec<ResponseStage>,
    pub pipelines: Vec<ResponseQueryPipeline>,
    pub pipelines_render_queues: Vec<ResponseQueryPipelineStageRenderQueue>,
    pub render_targets: Vec<ResponseQueryRenderTarget>,
    pub render_targets_color_data: Vec<ResponseQueryRenderTargetColorData>,
    pub meshes: Vec<ResponseQueryMesh>,
    pub meshes_data: Vec<ResponseQueryMeshData>,
    pub images: Vec<ResponseQueryImage>,
    pub images_data: Vec<ResponseQueryImageData>,
    pub materials: Vec<ResponseQueryMaterial>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTakeSnapshotText {
    pub stages: Vec<ResponseStage>,
    pub pipelines: Vec<ResponseQueryPipeline>,
    pub pipelines_render_queues: Vec<ResponseQueryPipelineStageRenderQueue>,
    pub render_targets: Vec<ResponseQueryRenderTarget>,
    pub render_targets_color_data: Vec<ResponseQueryRenderTargetColorDataText>,
    pub meshes: Vec<ResponseQueryMesh>,
    pub meshes_data: Vec<ResponseQueryMeshDataText>,
    pub images: Vec<ResponseQueryImage>,
    pub images_data: Vec<ResponseQueryImageDataText>,
    pub materials: Vec<ResponseQueryMaterial>,
}

pub enum Response {
    UnknownRequest(String),
    CheckPulse,
    TakeSnapshot(ResponseTakeSnapshot),
    RendererHasNoContext,
    ListStages(Vec<ResponseStage>),
    ListPipelines(Vec<PipelineId>),
    ListRenderTargets(Vec<RenderTargetId>),
    ListMeshes(Vec<MeshId>),
    ListImages(Vec<ImageId>),
    ListMaterials(Vec<MaterialId>),
    QueryPipeline(ResponseQueryPipeline),
    QueryPipelineResources(ResponseQueryPipelineResources),
    QueryPipelineStageRenderQueue(ResponseQueryPipelineStageRenderQueue),
    QueryPipelineStageRenderQueueResources(ResponseQueryPipelineStageRenderQueueResources),
    PipelineDoesNotExists(PipelineId),
    PipelineStageDoesNotExists(ResponsePipelineStageDoesNotExists),
    QueryRenderTarget(ResponseQueryRenderTarget),
    QueryRenderTargetColorData(ResponseQueryRenderTargetColorData),
    RenderTargetDoesNotExists(RenderTargetId),
    RenderTargetHasNoColorBuffer(ResponseRenderTargetHasNoColorBuffer),
    RenderTargetHasNoGpuResource(RenderTargetId),
    QueryMesh(ResponseQueryMesh),
    QueryMeshVertexData(ResponseQueryMeshVertexData),
    QueryMeshIndexData(ResponseQueryMeshIndexData),
    QueryMeshData(ResponseQueryMeshData),
    MeshDoesNotExists(MeshId),
    MeshVertexBufferDoesNotExists(ResponseMeshVertexBufferDoesNotExists),
    QueryImage(ResponseQueryImage),
    QueryImageData(ResponseQueryImageData),
    ImageDoesNotExists(ImageId),
    QueryMaterial(ResponseQueryMaterial),
    MaterialDoesNotExists(MaterialId),
}

macro_rules! message {
    ($id:literal, $text:expr, $binary:expr) => {
        SimpMessage::new(
            SimpMessageId::new($id, VERSION),
            serde_json::to_string($text).unwrap(),
            $binary,
        )
    };
    ($id:literal, $text:expr) => {
        SimpMessage::text(
            SimpMessageId::new($id, VERSION),
            serde_json::to_string($text).unwrap(),
        )
    };
    ($id:literal) => {
        SimpMessage::empty(SimpMessageId::new($id, VERSION))
    };
}

impl From<Response> for SimpMessage {
    fn from(value: Response) -> Self {
        match value {
            Response::UnknownRequest(data) => message!("UnknownRequest", &data),
            Response::CheckPulse => message!("CheckPulse"),
            Response::TakeSnapshot(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_take_snapshot(data, &mut bytes);
                message!("TakeSnapshot", &text, bytes)
            }
            Response::RendererHasNoContext => message!("RendererHasNoContext"),
            Response::ListStages(data) => message!("ListStages", &data),
            Response::ListPipelines(data) => message!("ListPipelines", &data),
            Response::ListRenderTargets(data) => message!("ListRenderTargets", &data),
            Response::ListMeshes(data) => message!("ListMeshes", &data),
            Response::ListImages(data) => message!("ListImages", &data),
            Response::ListMaterials(data) => message!("ListMaterials", &data),
            Response::QueryPipeline(data) => message!("QueryPipeline", &data),
            Response::QueryPipelineResources(data) => message!("QueryPipelineResources", &data),
            Response::QueryPipelineStageRenderQueue(data) => {
                message!("QueryPipelineStageRenderQueue", &data)
            }
            Response::QueryPipelineStageRenderQueueResources(data) => {
                message!("QueryPipelineStageRenderQueueResources", &data)
            }
            Response::PipelineDoesNotExists(data) => message!("PipelineDoesNotExists", &data),
            Response::PipelineStageDoesNotExists(data) => {
                message!("PipelineStageDoesNotExists", &data)
            }
            Response::QueryRenderTarget(data) => message!("QueryRenderTarget", &data),
            Response::QueryRenderTargetColorData(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_query_render_target_color_data(data, &mut bytes);
                message!("QueryRenderTargetColorData", &text, bytes)
            }
            Response::RenderTargetDoesNotExists(data) => {
                message!("RenderTargetDoesNotExists", &data)
            }
            Response::RenderTargetHasNoColorBuffer(data) => {
                message!("RenderTargetHasNoColorBuffer", &data)
            }
            Response::RenderTargetHasNoGpuResource(data) => {
                message!("RenderTargetHasNoGpuResource", &data)
            }
            Response::QueryMesh(data) => message!("QueryMesh", &data),
            Response::QueryMeshVertexData(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_query_mesh_vertex_data(data, &mut bytes);
                message!("QueryMeshVertexData", &text, bytes)
            }
            Response::QueryMeshIndexData(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_query_mesh_index_data(data, &mut bytes);
                message!("QueryMeshIndexData", &text, bytes)
            }
            Response::QueryMeshData(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_query_mesh_data(data, &mut bytes);
                message!("QueryMeshData", &text, bytes)
            }
            Response::MeshDoesNotExists(data) => message!("MeshDoesNotExists", &data),
            Response::MeshVertexBufferDoesNotExists(data) => {
                message!("MeshVertexBufferDoesNotExists", &data)
            }
            Response::QueryImage(data) => message!("QueryImage", &data),
            Response::QueryImageData(data) => {
                let mut bytes = vec![];
                let text = Response::prepare_query_image_data(data, &mut bytes);
                message!("QueryImageData", &text, bytes)
            }
            Response::ImageDoesNotExists(data) => message!("ImageDoesNotExists", &data),
            Response::QueryMaterial(data) => message!("QueryMaterial", &data),
            Response::MaterialDoesNotExists(data) => message!("MaterialDoesNotExists", &data),
        }
    }
}

macro_rules! write_bytes {
    ($source:expr, $target:expr) => {{
        let start = $target.len();
        $target.extend($source);
        start..$target.len()
    }};
}

impl Response {
    fn prepare_take_snapshot(
        data: ResponseTakeSnapshot,
        bytes: &mut Vec<u8>,
    ) -> ResponseTakeSnapshotText {
        ResponseTakeSnapshotText {
            stages: data.stages,
            pipelines: data.pipelines,
            pipelines_render_queues: data.pipelines_render_queues,
            render_targets: data.render_targets,
            render_targets_color_data: data
                .render_targets_color_data
                .into_iter()
                .map(|data| Self::prepare_query_render_target_color_data(data, bytes))
                .collect(),
            meshes: data.meshes,
            meshes_data: data
                .meshes_data
                .into_iter()
                .map(|data| Self::prepare_query_mesh_data(data, bytes))
                .collect(),
            images: data.images,
            images_data: data
                .images_data
                .into_iter()
                .map(|data| Self::prepare_query_image_data(data, bytes))
                .collect(),
            materials: data.materials,
        }
    }

    fn prepare_query_render_target_color_data(
        data: ResponseQueryRenderTargetColorData,
        bytes: &mut Vec<u8>,
    ) -> ResponseQueryRenderTargetColorDataText {
        ResponseQueryRenderTargetColorDataText {
            id: data.id,
            width: data.width,
            height: data.height,
            value_type: data.value_type,
            attachment_index: data.attachment_index,
            bytes_range: write_bytes!(data.bytes, bytes),
        }
    }

    fn prepare_query_mesh_vertex_data(
        data: ResponseQueryMeshVertexData,
        bytes: &mut Vec<u8>,
    ) -> ResponseQueryMeshVertexDataText {
        ResponseQueryMeshVertexDataText {
            id: data.id,
            buffer_index: data.buffer_index,
            layout: data.layout.to_owned(),
            bytes_range: write_bytes!(data.bytes, bytes),
        }
    }

    fn prepare_query_mesh_index_data(
        data: ResponseQueryMeshIndexData,
        bytes: &mut Vec<u8>,
    ) -> ResponseQueryMeshIndexDataText {
        ResponseQueryMeshIndexDataText {
            id: data.id,
            draw_mode: data.draw_mode,
            bytes_range: write_bytes!(data.bytes, bytes),
        }
    }

    fn prepare_query_mesh_data(
        data: ResponseQueryMeshData,
        bytes: &mut Vec<u8>,
    ) -> ResponseQueryMeshDataText {
        ResponseQueryMeshDataText {
            id: data.id,
            layout: data.layout.to_owned(),
            draw_mode: data.draw_mode,
            vertex_bytes_ranges: data
                .vertex_bytes
                .iter()
                .map(|data| write_bytes!(data, bytes))
                .collect(),
            index_bytes_range: write_bytes!(data.index_bytes, bytes),
        }
    }

    fn prepare_query_image_data(
        data: ResponseQueryImageData,
        bytes: &mut Vec<u8>,
    ) -> ResponseQueryImageDataText {
        ResponseQueryImageDataText {
            id: data.id,
            width: data.width,
            height: data.height,
            depth: data.depth,
            format: data.format,
            bytes_range: write_bytes!(data.bytes, bytes),
        }
    }
}
