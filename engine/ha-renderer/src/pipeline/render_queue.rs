use crate::{
    ha_renderer::{RenderStageResources, RenderStats},
    material::{
        common::{MaterialSignature, MaterialValue},
        MaterialDrawOptions, MaterialError, MaterialId,
    },
    math::*,
    mesh::{MeshDrawRange, MeshError, MeshId},
};
use glow::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Clone)]
pub enum RenderQueueError {
    QueueLimitReached(usize),
    MaterialDoesNotExist(MaterialId),
    MeshDoesNotExist(MeshId),
    Mesh(MeshId, Box<MeshError>),
    Material(MaterialId, Box<MaterialError>),
    NoMeshActive,
    NoMaterialActive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderCommand {
    SortingBarrier,
    Viewport(usize, usize, usize, usize),
    ActivateMaterial(MaterialId, MaterialSignature),
    OverrideUniform(Cow<'static, str>, MaterialValue),
    ResetUniform(Cow<'static, str>),
    ResetUniforms,
    ApplyDrawOptions(MaterialDrawOptions),
    ActivateMesh(MeshId),
    DrawMesh(MeshDrawRange),
    /// (x, y, width, height)
    Scissor(Option<(usize, usize, usize, usize)>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GroupOrder(usize, usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderQueueSize {
    Limited(usize),
    Growable(usize),
}

impl Default for RenderQueueSize {
    fn default() -> Self {
        Self::Growable(1024)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderQueue {
    size: RenderQueueSize,
    commands: Vec<(GroupOrder, RenderCommand)>,
    pub persistent: bool,
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new(Default::default(), false)
    }
}

impl RenderQueue {
    pub fn new(size: RenderQueueSize, persistent: bool) -> Self {
        Self {
            size,
            commands: Vec::with_capacity(match size {
                RenderQueueSize::Limited(size) => size,
                RenderQueueSize::Growable(size) => size,
            }),
            persistent,
        }
    }

    pub fn size(&self) -> RenderQueueSize {
        self.size
    }

    pub fn auto_recorder(&mut self, initial_group: Option<usize>) -> RenderQueueAutoRecorder {
        RenderQueueAutoRecorder::new(self, initial_group.unwrap_or_default())
    }

    pub fn record(
        &mut self,
        group: usize,
        order: usize,
        command: RenderCommand,
    ) -> Result<(), RenderQueueError> {
        match self.size {
            RenderQueueSize::Limited(size) => {
                if self.commands.len() >= size {
                    return Err(RenderQueueError::QueueLimitReached(size));
                }
            }
            RenderQueueSize::Growable(resize) => {
                if self.commands.len() % resize == 0 {
                    self.commands.reserve(resize);
                }
            }
        }
        self.commands.push((GroupOrder(group, order), command));
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn commands(&self) -> &[(GroupOrder, RenderCommand)] {
        &self.commands
    }

    pub fn iter(&self) -> impl Iterator<Item = &RenderCommand> {
        self.commands.iter().map(|(_, command)| command)
    }

    pub fn move_into(&mut self, other: &mut Self) -> Result<(), RenderQueueError> {
        for (GroupOrder(group, order), command) in self.commands.drain(..) {
            other.record(group, order, command)?;
        }
        Ok(())
    }

    pub fn clone_into(&self, other: &mut Self) -> Result<(), RenderQueueError> {
        for (GroupOrder(group, order), command) in &self.commands {
            other.record(*group, *order, command.to_owned())?;
        }
        Ok(())
    }

    pub fn remap_groups<F>(&mut self, mut f: F)
    where
        F: FnMut(usize) -> usize,
    {
        for (GroupOrder(group, _), _) in &mut self.commands {
            *group = f(*group);
        }
    }

    pub fn sort_by_group_order(&mut self, stable: bool) {
        let mut start = 0;
        for index in 0..self.commands.len() {
            if matches!(&self.commands[index].1, RenderCommand::SortingBarrier)
                || index == self.commands.len() - 1
            {
                if index - start > 1 {
                    if stable {
                        self.commands[start..index].sort_by(|a, b| a.0.cmp(&b.0));
                    } else {
                        self.commands[start..index].sort_unstable_by(|a, b| a.0.cmp(&b.0));
                    }
                }
                start = index + 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn execute(
        &mut self,
        context: &Context,
        resources: &RenderStageResources<'_>,
        stats: &mut RenderStats,
    ) -> Result<(), RenderQueueError> {
        let result = self.execute_inner(context, resources, stats);
        if !self.persistent {
            self.commands.clear();
        }
        result
    }

    fn execute_inner(
        &mut self,
        context: &Context,
        resources: &RenderStageResources<'_>,
        stats: &mut RenderStats,
    ) -> Result<(), RenderQueueError> {
        let mut current_material = None;
        let mut current_mesh = None;
        let mut current_uniforms = HashMap::<&str, &MaterialValue>::with_capacity(32);
        let mut last_uniforms = HashMap::<&str, &MaterialValue>::with_capacity(32);
        let mut current_scissor = None;
        for (_, command) in &self.commands {
            match command {
                RenderCommand::SortingBarrier => {}
                RenderCommand::Viewport(x, y, w, h) => unsafe {
                    context.viewport(*x as _, *y as _, *w as _, *h as _);
                },
                RenderCommand::ActivateMaterial(id, signature) => {
                    if current_material
                        .map(|(cid, _, _)| cid == id)
                        .unwrap_or_default()
                    {
                        continue;
                    }
                    match resources.materials.get(*id) {
                        Some(material) => {
                            match material.activate(signature, context, resources, stats) {
                                Ok(_) => {
                                    current_uniforms.clear();
                                    last_uniforms.clear();
                                    current_material = Some((id, signature, material));
                                }
                                Err(error) => {
                                    return Err(RenderQueueError::Material(*id, Box::new(error)))
                                }
                            }
                        }
                        None => return Err(RenderQueueError::MaterialDoesNotExist(*id)),
                    }
                }
                RenderCommand::OverrideUniform(name, value) => {
                    current_uniforms.insert(name.as_ref(), value);
                }
                RenderCommand::ResetUniform(name) => {
                    current_uniforms.remove(name.as_ref());
                }
                RenderCommand::ResetUniforms => {
                    current_uniforms.clear();
                }
                RenderCommand::ApplyDrawOptions(draw_options) => {
                    draw_options.apply(context, stats);
                }
                RenderCommand::ActivateMesh(id) => {
                    if current_mesh.map(|(cid, _)| cid == id).unwrap_or_default() {
                        continue;
                    }
                    match resources.meshes.get(*id) {
                        Some(mesh) => match mesh.activate(context, stats) {
                            Ok(_) => current_mesh = Some((id, mesh)),
                            Err(error) => return Err(RenderQueueError::Mesh(*id, Box::new(error))),
                        },
                        None => return Err(RenderQueueError::MeshDoesNotExist(*id)),
                    }
                }
                RenderCommand::DrawMesh(draw_range) => {
                    let (mesh_id, mesh) = match current_mesh {
                        Some((id, mesh)) => (id, mesh),
                        None => return Err(RenderQueueError::NoMeshActive),
                    };
                    let (material_id, signature, material) = match current_material {
                        Some((id, signature, material)) => (id, signature, material),
                        None => return Err(RenderQueueError::NoMaterialActive),
                    };
                    for (name, current_value) in &current_uniforms {
                        if last_uniforms
                            .get(name)
                            .map(|last_value| current_value != last_value)
                            .unwrap_or(true)
                        {
                            if let Err(error) = material.submit_uniform(
                                signature,
                                name,
                                current_value,
                                context,
                                resources,
                                stats,
                            ) {
                                return Err(RenderQueueError::Material(
                                    *material_id,
                                    Box::new(error),
                                ));
                            }
                        }
                    }
                    for name in last_uniforms.keys() {
                        let name: &str = name;
                        if !current_uniforms.contains_key(name) {
                            if let Some(default_value) = material.default_values.get(name) {
                                if let Err(error) = material.submit_uniform(
                                    signature,
                                    name,
                                    default_value,
                                    context,
                                    resources,
                                    stats,
                                ) {
                                    return Err(RenderQueueError::Material(
                                        *material_id,
                                        Box::new(error),
                                    ));
                                }
                            }
                        }
                    }
                    if let Err(error) = mesh.draw(draw_range.clone(), context, stats) {
                        return Err(RenderQueueError::Mesh(*mesh_id, Box::new(error)));
                    }
                    last_uniforms.clear();
                    last_uniforms.reserve(current_uniforms.len());
                    for (key, value) in &current_uniforms {
                        last_uniforms.insert(key, value);
                    }
                }
                RenderCommand::Scissor(rect) => {
                    match (current_scissor.is_some(), rect) {
                        (false, Some(rect)) => unsafe {
                            context.enable(SCISSOR_TEST);
                            context.scissor(rect.0 as _, rect.1 as _, rect.2 as _, rect.3 as _);
                        },
                        (true, Some(rect)) => unsafe {
                            context.scissor(rect.0 as _, rect.1 as _, rect.2 as _, rect.3 as _);
                        },
                        (true, None) => unsafe {
                            context.disable(SCISSOR_TEST);
                        },
                        (false, None) => {}
                    }
                    current_scissor = *rect;
                }
            }
        }
        Ok(())
    }
}

pub struct RenderQueueAutoRecorder<'a> {
    group: usize,
    order: usize,
    queue: &'a mut RenderQueue,
}

impl<'a> RenderQueueAutoRecorder<'a> {
    fn new(queue: &'a mut RenderQueue, initial_group: usize) -> Self {
        Self {
            group: initial_group,
            order: 0,
            queue,
        }
    }

    pub fn group_order(&self) -> (usize, usize) {
        (self.group, self.order)
    }

    pub fn next_group(&mut self) -> usize {
        self.order = 0;
        self.group += 1;
        self.group
    }

    pub fn record(&mut self, command: RenderCommand) -> Result<usize, RenderQueueError> {
        self.order += 1;
        self.queue.record(self.group, self.order, command)?;
        Ok(self.order)
    }
}
