use crate::{
    material::common::{MaterialMeshSignature, MaterialRenderTargetSignature, MaterialSignature},
    math::*,
    mesh::VertexLayout,
    pipeline::render_queue::{RenderQueue, RenderQueueSize},
    TagFilters,
};
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub enum StageError {}

#[derive(Debug)]
pub struct StageProcessInfo {
    pub width: usize,
    pub height: usize,
    pub transform_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub material_render_target_signature: MaterialRenderTargetSignature,
    pub domain: Option<String>,
    pub filters: TagFilters,
}

impl StageProcessInfo {
    pub fn make_material_signature(&self, vertex_layout: &VertexLayout) -> MaterialSignature {
        MaterialSignature::new(
            MaterialMeshSignature::new(vertex_layout),
            self.material_render_target_signature.to_owned(),
            self.domain.to_owned(),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageQueueSorting {
    None,
    Unstable,
    Stable,
}

impl Default for StageQueueSorting {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearSettings {
    pub color: Option<Rgba>,
    pub depth: bool,
    pub stencil: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StageDescriptor {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) filters: TagFilters,
    #[serde(default)]
    pub(crate) render_target: String,
    #[serde(default)]
    pub(crate) domain: Option<String>,
    #[serde(default)]
    pub(crate) queue_size: RenderQueueSize,
    #[serde(default)]
    pub(crate) queue_persistent: bool,
    #[serde(default)]
    pub(crate) queue_sorting: StageQueueSorting,
    #[serde(default)]
    pub(crate) clear_settings: ClearSettings,
}

impl StageDescriptor {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            filters: Default::default(),
            render_target: Default::default(),
            domain: None,
            queue_size: Default::default(),
            queue_persistent: false,
            queue_sorting: Default::default(),
            clear_settings: Default::default(),
        }
    }

    pub fn filters(mut self, filters: TagFilters) -> Self {
        self.filters = filters;
        self
    }

    pub fn render_target(mut self, name: impl ToString) -> Self {
        self.render_target = name.to_string();
        self
    }

    pub fn domain(mut self, name: impl ToString) -> Self {
        self.domain = Some(name.to_string());
        self
    }

    pub fn queue_size(mut self, size: RenderQueueSize) -> Self {
        self.queue_size = size;
        self
    }

    pub fn queue_persistent(mut self, mode: bool) -> Self {
        self.queue_persistent = mode;
        self
    }

    pub fn queue_sorting(mut self, sorting: StageQueueSorting) -> Self {
        self.queue_sorting = sorting;
        self
    }

    pub fn clear_settings(mut self, settings: ClearSettings) -> Self {
        self.clear_settings = settings;
        self
    }
}

#[derive(Debug)]
pub struct StageDetailedInfo {
    pub queue_sorting: StageQueueSorting,
    pub filters: TagFilters,
    pub render_target: String,
    pub domain: Option<String>,
    pub clear_settings: ClearSettings,
}

#[derive(Debug)]
pub(crate) struct Stage {
    pub type_id: TypeId,
    pub render_queue: Arc<RwLock<RenderQueue>>,
    pub queue_sorting: StageQueueSorting,
    pub filters: TagFilters,
    pub render_target: String,
    pub domain: Option<String>,
    pub clear_settings: ClearSettings,
}

impl Stage {
    pub fn detailed_info(&self) -> StageDetailedInfo {
        StageDetailedInfo {
            queue_sorting: self.queue_sorting,
            filters: self.filters.clone(),
            render_target: self.render_target.clone(),
            domain: self.domain.clone(),
            clear_settings: self.clear_settings,
        }
    }
}
