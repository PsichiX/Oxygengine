pub mod render_queue;
pub mod stage;

use crate::{
    math::*,
    pipeline::{render_queue::*, stage::*},
    render_target::*,
};
use core::{id::ID, utils::TagFilters};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PipelineError {
    StageNotFound(String),
    DescriptorNotFound(String),
    CouldNotCreateRenderTarget(RenderTargetError),
    CouldNotDestroyRenderTarget(RenderTargetError),
}

pub type PipelineId = ID<Pipeline>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PipelineDescriptor {
    #[serde(default)]
    pub(crate) filters: TagFilters,
    #[serde(default)]
    pub(crate) stages: Vec<StageDescriptor>,
    #[serde(default)]
    pub(crate) render_targets: HashMap<String, RenderTargetDescriptor>,
}

impl PipelineDescriptor {
    pub fn filters(mut self, filters: TagFilters) -> Self {
        self.filters = filters;
        self
    }

    pub fn stage(mut self, data: StageDescriptor) -> Self {
        self.stages.push(data);
        self
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    pub fn debug_stage(mut self, data: StageDescriptor) -> Self {
        #[cfg(debug_assertions)]
        {
            self.stage(data)
        }
        #[cfg(not(debug_assertions))]
        {
            self
        }
    }

    pub fn render_target(mut self, name: impl ToString, data: RenderTargetDescriptor) -> Self {
        self.render_targets.insert(name.to_string(), data);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineDetailedInfo {
    pub stages: Vec<StageDetailedInfo>,
    pub render_targets: HashMap<String, (RenderTargetDescriptor, RenderTargetId)>,
}

#[derive(Debug, Default)]
pub struct Pipeline {
    pub(crate) stages: Vec<Stage>,
    pub(crate) render_targets: HashMap<String, (RenderTargetDescriptor, RenderTargetId)>,
}

impl Pipeline {
    pub fn detailed_info(&self) -> PipelineDetailedInfo {
        PipelineDetailedInfo {
            stages: self.stages.iter().map(|s| s.detailed_info()).collect(),
            render_targets: self.render_targets.clone(),
        }
    }

    pub fn render_targets(&self) -> impl Iterator<Item = RenderTargetId> + '_ {
        self.render_targets.values().map(|(_, id)| *id)
    }

    pub fn stages_count(&self) -> usize {
        self.stages.len()
    }

    pub fn cloned_stage_render_queue(&self, index: usize) -> Option<RenderQueue> {
        self.stages.get(index).and_then(|stage| {
            stage
                .render_queue
                .try_read()
                .ok()
                .map(|queue| queue.clone())
        })
    }
}
