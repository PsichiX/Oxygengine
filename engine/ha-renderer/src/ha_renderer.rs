use crate::{
    image::*,
    material::{common::*, *},
    math::*,
    mesh::*,
    pipeline::{render_queue::*, stage::*, *},
    platform::HaPlatformInterface,
    render_target::*,
    resources::material_library::*,
    Error, HasContextResources, Resources,
};
use core::{utils::StringSequence, Ignite};
use glow::*;
use serde::{Deserialize, Serialize};
use std::{
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub trait HaRendererErrorReporter: Send + Sync {
    fn on_report(&self, error: Error);
}

impl HaRendererErrorReporter for () {
    fn on_report(&self, _: Error) {}
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LoggerHaRendererErrorReporter;

impl HaRendererErrorReporter for LoggerHaRendererErrorReporter {
    fn on_report(&self, error: Error) {
        oxygengine_core::error!("HA Renderer error: {:#?}", error);
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum PipelineSource {
    Registry(String),
    Descriptor(PipelineDescriptor),
}

impl Default for PipelineSource {
    fn default() -> Self {
        Self::Registry(Default::default())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RenderStats {
    pub draw_calls: usize,
    pub mesh_changes: usize,
    pub material_changes: usize,
    pub uniform_changes: usize,
    pub sampler_changes: usize,
    pub state_changes: usize,
}

pub struct RenderStageResources<'a> {
    pub render_targets: &'a Resources<RenderTarget>,
    pub images: &'a Resources<Image>,
    pub virtual_images: &'a Resources<VirtualImage>,
    pub meshes: &'a Resources<Mesh>,
    pub virtual_meshes: &'a Resources<VirtualMesh>,
    pub materials: &'a Resources<Material>,
}

impl<'a> RenderStageResources<'a> {
    pub fn mesh_by_ref(&self, reference: &MeshReference) -> Option<&Mesh> {
        match reference {
            MeshReference::Id(id) => self.meshes.get(*id),
            MeshReference::VirtualId { owner, .. } => {
                if let Some(virtual_mesh) = self.virtual_meshes.get(*owner) {
                    return self.meshes.get(virtual_mesh.source());
                }
                None
            }
            _ => None,
        }
    }

    pub fn image_by_ref(&self, reference: &ImageReference) -> Option<&Image> {
        match reference {
            ImageReference::Id(id) => self.images.get(*id),
            _ => None,
        }
    }

    pub fn image_handle_by_ref(
        &self,
        reference: &ImageReference,
    ) -> Option<<Context as HasContext>::Texture> {
        match reference {
            ImageReference::Id(id) => {
                if let Some(image) = self.images.get(*id) {
                    if let Some(resources) = image.resources(self) {
                        return Some(resources.handle);
                    }
                }
                None
            }
            ImageReference::VirtualId { owner, .. } => {
                if let Some(virtual_image) = self.virtual_images.get(*owner) {
                    match virtual_image.source() {
                        VirtualImageSource::Image(id) => {
                            if let Some(image) = self.images.get(*id) {
                                if let Some(resources) = image.resources(self) {
                                    return Some(resources.handle);
                                }
                            }
                        }
                        VirtualImageSource::RenderTargetDepthStencil(id) => {
                            if let Some(render_target) = self.render_targets.get(*id) {
                                return render_target.depth_stencil_texture_handle();
                            }
                        }
                        VirtualImageSource::RenderTargetColor(id, name) => {
                            if let Some(render_target) = self.render_targets.get(*id) {
                                return render_target.color_texture_handle(name);
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn draw_range_by_ref(&self, reference: &MeshReference) -> Option<MeshDrawRange> {
        match reference {
            MeshReference::Id(_) => Some(MeshDrawRange::All),
            MeshReference::VirtualId { owner, id } => {
                if let Some(virtual_mesh) = self.virtual_meshes.get(*owner) {
                    return virtual_mesh.mesh_range(*id).map(MeshDrawRange::Range);
                }
                None
            }
            _ => None,
        }
    }

    pub fn material_by_ref(&self, reference: &MaterialReference) -> Option<&Material> {
        match reference {
            MaterialReference::Id(id) => self.materials.get(*id),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct HaRendererInfo {
    pub pipeline_registry: Vec<String>,
    pub pipelines: Vec<PipelineId>,
    pub render_targets: Vec<RenderTargetId>,
    pub meshes: Vec<MeshId>,
    pub images: Vec<ImageId>,
    pub virtual_images: Vec<VirtualImageId>,
    pub materials: HashMap<MaterialId, Vec<MaterialSignature>>,
    pub stats: RenderStats,
}

#[derive(Debug)]
pub struct HaRendererDetailedInfo {
    pub pipeline_registry: Option<HashMap<String, PipelineDescriptor>>,
    pub pipelines: Option<HashMap<PipelineId, PipelineDetailedInfo>>,
    pub render_targets: Option<HashMap<RenderTargetId, RenderTargetDetailedInfo>>,
    pub images: Option<HashMap<ImageId, ImageDetailedInfo>>,
    pub virtual_images: Option<HashMap<VirtualImageId, VirtualImageDetailedInfo>>,
    pub meshes: Option<HashMap<MeshId, MeshDetailedInfo>>,
    pub virtual_meshes: Option<HashMap<VirtualMeshId, VirtualMeshDetailedInfo>>,
    pub materials: Option<HashMap<MaterialId, MaterialDetailedInfo>>,
    pub stats: RenderStats,
}

#[derive(Debug, Clone)]
pub struct HaRendererDetailedInfoFilter {
    pub pipeline_registry: bool,
    pub pipelines: bool,
    pub render_targets: bool,
    pub images: bool,
    pub virtual_images: bool,
    pub meshes: bool,
    pub virtual_meshes: bool,
    pub materials: bool,
}

impl HaRendererDetailedInfoFilter {
    pub fn all() -> Self {
        Self {
            pipeline_registry: true,
            pipelines: true,
            render_targets: true,
            images: true,
            virtual_images: true,
            meshes: true,
            virtual_meshes: true,
            materials: true,
        }
    }

    pub fn empty() -> Self {
        Self {
            pipeline_registry: false,
            pipelines: false,
            render_targets: false,
            images: false,
            virtual_images: false,
            meshes: false,
            virtual_meshes: false,
            materials: false,
        }
    }
}

pub struct HaRenderer {
    pub(crate) platform_interface: Box<dyn HaPlatformInterface + Send + Sync>,
    stage_registry: HashMap<String, (TypeId, String)>,
    pipeline_registry: HashMap<String, PipelineDescriptor>,
    pub(crate) pipelines: HashMap<PipelineId, Pipeline>,
    pub(crate) render_targets: Resources<RenderTarget>,
    meshes: Resources<Mesh>,
    images: Resources<Image>,
    pub virtual_images: Resources<VirtualImage>,
    pub virtual_meshes: Resources<VirtualMesh>,
    pub(crate) materials: Resources<Material>,
    cached_signatures: HashSet<MaterialSignature>,
    dirty_signatures: bool,
    added_materials: HashSet<MaterialId>,
    pub(crate) stats_cache: RenderStats,
    pub(crate) error_reporter: Box<dyn HaRendererErrorReporter>,
}

impl std::fmt::Debug for HaRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HaRenderer")
            .field("stage_registry", &self.stage_registry)
            .field("pipeline_registry", &self.pipeline_registry)
            .field("pipelines", &self.pipelines)
            .field("render_targets", &self.render_targets)
            .field("meshes", &self.meshes)
            .field("images", &self.images)
            .field("virtual_meshes", &self.virtual_meshes)
            .field("virtual_images", &self.virtual_images)
            .field("materials", &self.materials)
            .field("cached_signatures", &self.cached_signatures)
            .field("dirty_signatures", &self.dirty_signatures)
            .field("added_materials", &self.added_materials)
            .field("stats_cache", &self.stats_cache)
            .finish()
    }
}

impl HaRenderer {
    pub fn new<PI>(platform_interface: PI) -> Self
    where
        PI: HaPlatformInterface + Send + Sync + 'static,
    {
        Self::new_raw(Box::new(platform_interface))
    }

    pub fn new_raw(platform_interface: Box<dyn HaPlatformInterface + Send + Sync>) -> Self {
        Self {
            platform_interface,
            stage_registry: Default::default(),
            pipeline_registry: Default::default(),
            pipelines: Default::default(),
            render_targets: Default::default(),
            images: Default::default(),
            virtual_images: Default::default(),
            meshes: Default::default(),
            virtual_meshes: Default::default(),
            materials: Default::default(),
            cached_signatures: Default::default(),
            dirty_signatures: true,
            added_materials: Default::default(),
            stats_cache: Default::default(),
            error_reporter: Box::new(()),
        }
    }

    #[inline]
    pub fn screen_size(&self) -> (usize, usize) {
        self.platform_interface.screen_size()
    }

    #[inline]
    pub fn interface(&self) -> &dyn HaPlatformInterface {
        &*self.platform_interface
    }

    #[inline]
    pub fn stats(&self) -> &RenderStats {
        &self.stats_cache
    }

    pub fn info(&self) -> HaRendererInfo {
        HaRendererInfo {
            pipeline_registry: self.pipeline_registry.keys().cloned().collect(),
            pipelines: self.pipelines.keys().cloned().collect(),
            render_targets: self.render_targets.ids().collect(),
            meshes: self.meshes.ids().collect(),
            images: self.images.ids().collect(),
            virtual_images: self.virtual_images.ids().collect(),
            materials: self
                .materials
                .iter()
                .map(|(k, v)| (k.to_owned(), v.versions().cloned().collect()))
                .collect(),
            stats: self.stats_cache.clone(),
        }
    }

    pub fn detailed_info(&self, filter: &HaRendererDetailedInfoFilter) -> HaRendererDetailedInfo {
        HaRendererDetailedInfo {
            pipeline_registry: if filter.pipeline_registry {
                Some(self.pipeline_registry.clone())
            } else {
                None
            },
            pipelines: if filter.pipelines {
                Some(
                    self.pipelines
                        .iter()
                        .map(|(k, v)| (*k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            render_targets: if filter.render_targets {
                Some(
                    self.render_targets
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            images: if filter.images {
                Some(
                    self.images
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            virtual_images: if filter.images {
                Some(
                    self.virtual_images
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            meshes: if filter.meshes {
                Some(
                    self.meshes
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            virtual_meshes: if filter.virtual_meshes {
                Some(
                    self.virtual_meshes
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            materials: if filter.materials {
                Some(
                    self.materials
                        .iter()
                        .map(|(k, v)| (k, v.detailed_info()))
                        .collect(),
                )
            } else {
                None
            },
            stats: self.stats_cache.clone(),
        }
    }

    pub fn register_stage<T: 'static>(&mut self, id: &str) {
        self.stage_registry.insert(
            id.to_owned(),
            (TypeId::of::<T>(), type_name::<T>().to_owned()),
        );
    }

    pub fn with_stage<T: 'static>(mut self, id: &str) -> Self {
        self.register_stage::<T>(id);
        self
    }

    pub fn stages(&self) -> impl Iterator<Item = (&str, TypeId, &str)> {
        self.stage_registry
            .iter()
            .map(|(n, (t, tn))| (n.as_str(), *t, tn.as_str()))
    }

    pub fn unregister_stage(&mut self, id: &str) {
        self.stage_registry.remove(id);
    }

    pub fn register_pipeline(&mut self, id: impl ToString, data: PipelineDescriptor) {
        self.pipeline_registry.insert(id.to_string(), data);
    }

    pub fn with_pipeline(mut self, id: impl ToString, data: PipelineDescriptor) -> Self {
        self.register_pipeline(id, data);
        self
    }

    pub fn unregister_pipeline(&mut self, id: &str) {
        self.pipeline_registry.remove(id);
    }

    pub fn add_pipeline(&mut self, source: PipelineSource) -> Result<PipelineId, PipelineError> {
        let data = match source {
            PipelineSource::Registry(name) => match self.pipeline_registry.get(&name) {
                Some(descriptor) => descriptor.to_owned(),
                None => return Err(PipelineError::DescriptorNotFound(name)),
            },
            PipelineSource::Descriptor(descriptor) => descriptor,
        };
        let mut stages = Vec::with_capacity(data.stages.len());
        for stage in data.stages {
            if let Some((type_id, _)) = self.stage_registry.get(&stage.name) {
                stages.push(Stage {
                    type_id: *type_id,
                    render_queue: Arc::new(RwLock::new(RenderQueue::new(
                        stage.queue_size,
                        stage.queue_persistent,
                    ))),
                    queue_sorting: stage.queue_sorting,
                    render_target: stage.render_target,
                    filters: stage.filters.combine(&data.filters),
                    domain: stage.domain,
                    clear_settings: stage.clear_settings,
                });
            } else {
                return Err(PipelineError::StageNotFound(stage.name.to_owned()));
            }
        }
        let mut render_targets = HashMap::with_capacity(data.render_targets.len());
        for (key, data) in data.render_targets {
            let render_target = match &data {
                RenderTargetDescriptor::Main => match RenderTarget::main() {
                    Ok(render_target) => render_target,
                    Err(error) => return Err(PipelineError::CouldNotCreateRenderTarget(error)),
                },
                RenderTargetDescriptor::Custom {
                    buffers,
                    width,
                    height,
                } => RenderTarget::new(buffers.to_owned(), *width, *height),
            };
            let id = match self.add_render_target(render_target) {
                Ok(id) => id,
                Err(error) => return Err(PipelineError::CouldNotCreateRenderTarget(error)),
            };
            render_targets.insert(key, (data, id));
        }
        let id = PipelineId::new();
        self.pipelines.insert(
            id,
            Pipeline {
                stages,
                render_targets,
            },
        );
        self.dirty_signatures = true;
        Ok(id)
    }

    pub fn remove_pipeline(&mut self, id: PipelineId) -> Result<(), PipelineError> {
        if let Some(data) = self.pipelines.remove(&id) {
            for (_, (_, id)) in data.render_targets {
                if let Err(error) = self.remove_render_target(id) {
                    return Err(PipelineError::CouldNotDestroyRenderTarget(error));
                }
            }
            self.dirty_signatures = true;
        }
        Ok(())
    }

    pub fn pipelines(&self) -> impl Iterator<Item = PipelineId> + '_ {
        self.pipelines.keys().copied()
    }

    pub fn pipeline(&self, id: PipelineId) -> Option<&Pipeline> {
        self.pipelines.get(&id)
    }

    pub fn render_targets(&self) -> &Resources<RenderTarget> {
        &self.render_targets
    }

    pub fn add_render_target(
        &mut self,
        mut data: RenderTarget,
    ) -> Result<RenderTargetId, RenderTargetError> {
        if let Some(context) = self.platform_interface.context() {
            data.context_initialize(context)?;
        }
        Ok(self.render_targets.add(data))
    }

    pub fn remove_render_target(&mut self, id: RenderTargetId) -> Result<(), RenderTargetError> {
        if let Some(mut data) = self.render_targets.remove(id) {
            if let Some(context) = self.platform_interface.context() {
                data.context_release(context)?;
            }
        }
        Ok(())
    }

    pub fn render_target(&self, id: RenderTargetId) -> Option<&RenderTarget> {
        self.render_targets.get(id)
    }

    pub fn render_target_mut(&mut self, id: RenderTargetId) -> Option<&mut RenderTarget> {
        self.render_targets.get_mut(id)
    }

    pub fn meshes(&self) -> &Resources<Mesh> {
        &self.meshes
    }

    pub fn add_mesh(&mut self, mut data: Mesh) -> Result<MeshId, MeshError> {
        if let Some(context) = self.platform_interface.context() {
            data.context_initialize(context)?;
        }
        let id = self.meshes.add(data);
        self.dirty_signatures = true;
        Ok(id)
    }

    pub fn remove_mesh(&mut self, id: MeshId) -> Result<(), MeshError> {
        if let Some(mut data) = self.meshes.remove(id) {
            if let Some(context) = self.platform_interface.context() {
                data.context_release(context)?;
            }
            self.dirty_signatures = true;
        }
        Ok(())
    }

    pub fn mesh(&self, id: MeshId) -> Option<&Mesh> {
        self.meshes.get(id)
    }

    pub fn mesh_mut(&mut self, id: MeshId) -> Option<&mut Mesh> {
        self.meshes.get_mut(id)
    }

    pub fn images(&self) -> &Resources<Image> {
        &self.images
    }

    pub fn add_image(&mut self, mut data: Image) -> Result<ImageId, ImageError> {
        if let Some(context) = self.platform_interface.context() {
            data.context_initialize(context)?;
        }
        Ok(self.images.add(data))
    }

    pub fn remove_image(&mut self, id: ImageId) -> Result<(), ImageError> {
        if let Some(mut data) = self.images.remove(id) {
            if let Some(context) = self.platform_interface.context() {
                data.context_release(context)?;
            }
        }
        Ok(())
    }

    pub fn image(&self, id: ImageId) -> Option<&Image> {
        self.images.get(id)
    }

    pub fn image_mut(&mut self, id: ImageId) -> Option<&mut Image> {
        self.images.get_mut(id)
    }

    pub fn materials(&self) -> &Resources<Material> {
        &self.materials
    }

    pub fn add_material(&mut self, mut data: Material) -> Result<MaterialId, MaterialError> {
        if let Some(context) = self.platform_interface.context() {
            data.context_initialize(context)?;
        }
        let id = self.materials.add(data);
        self.added_materials.insert(id);
        Ok(id)
    }

    pub fn remove_material(&mut self, id: MaterialId) -> Result<(), MaterialError> {
        if let Some(mut data) = self.materials.remove(id) {
            if let Some(context) = self.platform_interface.context() {
                data.context_release(context)?;
            }
        }
        Ok(())
    }

    pub fn material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    #[inline]
    pub fn error_reporter(&self) -> &dyn HaRendererErrorReporter {
        &*self.error_reporter
    }

    #[inline]
    pub fn set_error_reporter<R>(&mut self, error_reporter: R)
    where
        R: HaRendererErrorReporter + 'static,
    {
        self.error_reporter = Box::new(error_reporter);
    }

    #[inline]
    pub fn report_error(&self, error: impl Into<Error>) {
        self.error_reporter.on_report(error.into());
    }

    pub(crate) fn maintain_platform_interface(&mut self) {
        let result = self.platform_interface.maintain();
        if let Some(ref context) = result.context_lost {
            for (id, render_target) in self.render_targets.iter_mut() {
                if let Err(error) = render_target.context_release(context) {
                    self.error_reporter
                        .on_report(Error::RenderTarget(id, error));
                }
            }
            for (id, mesh) in self.meshes.iter_mut() {
                if let Err(error) = mesh.context_release(context) {
                    self.error_reporter.on_report(Error::Mesh(id, error));
                }
            }
            for (id, image) in self.images.iter_mut() {
                if let Err(error) = image.context_release(context) {
                    self.error_reporter.on_report(Error::Image(id, error));
                }
            }
            for (id, material) in self.materials.iter_mut() {
                if let Err(error) = material.context_release(context) {
                    self.error_reporter.on_report(Error::Material(id, error));
                }
            }
        }
        if let Some(context) = result.context_acquired {
            for (id, render_target) in self.render_targets.iter_mut() {
                if let Err(error) = render_target.context_initialize(context) {
                    self.error_reporter
                        .on_report(Error::RenderTarget(id, error));
                }
            }
            for (id, mesh) in self.meshes.iter_mut() {
                if let Err(error) = mesh.context_initialize(context) {
                    self.error_reporter.on_report(Error::Mesh(id, error));
                }
            }
            for (id, image) in self.images.iter_mut() {
                if let Err(error) = image.context_initialize(context) {
                    self.error_reporter.on_report(Error::Image(id, error));
                }
            }
            for (id, material) in self.materials.iter_mut() {
                if let Err(error) = material.context_initialize(context) {
                    self.error_reporter.on_report(Error::Material(id, error));
                }
            }
        }
        if let Some((width, height)) = result.screen_resized {
            if let Some(context) = self.platform_interface.context() {
                for (id, render_target) in self.render_targets.iter_mut() {
                    if let Err(error) = render_target.screen_resize(context, width, height) {
                        self.error_reporter
                            .on_report(Error::RenderTarget(id, error));
                    }
                }
            }
        }
    }

    pub(crate) fn maintain_render_targets(&mut self) {
        let context = match self.platform_interface.context() {
            Some(context) => context,
            None => return,
        };
        let (width, height) = self.platform_interface.screen_size();
        for (id, render_target) in self.render_targets.iter_mut() {
            if let Err(error) = render_target.screen_resize(context, width, height) {
                self.error_reporter
                    .on_report(Error::RenderTarget(id, error));
            }
        }
    }

    pub(crate) fn maintain_images(&mut self) {
        let context = match self.platform_interface.context() {
            Some(context) => context,
            None => return,
        };
        for (id, image) in self.images.iter_mut() {
            if let Err(error) = image.maintain(context) {
                self.error_reporter.on_report(Error::Image(id, error));
            }
        }
    }

    pub(crate) fn maintain_meshes(&mut self) {
        let context = match self.platform_interface.context() {
            Some(context) => context,
            None => return,
        };
        for (id, mesh) in self.meshes.iter_mut() {
            if let Err(error) = mesh.maintain(context) {
                self.error_reporter.on_report(Error::Mesh(id, error));
            }
        }
    }

    pub(crate) fn maintain_materials(
        &mut self,
        library: &MaterialLibrary,
        fragment_high_precision_support: bool,
    ) {
        let context = match self.platform_interface.context() {
            Some(context) => context,
            None => return,
        };
        if self.dirty_signatures {
            let mesh_signatures_middlewares = self
                .meshes
                .resources()
                .map(|mesh| {
                    (
                        MaterialMeshSignature::new(mesh.layout()),
                        StringSequence::new(mesh.layout().middlewares()),
                    )
                })
                .collect::<HashMap<_, _>>();
            let count = self.cached_signatures.len();
            let old = std::mem::replace(&mut self.cached_signatures, HashSet::with_capacity(count));
            if !mesh_signatures_middlewares.is_empty() {
                for pipeline in self.pipelines.values() {
                    for stage in &pipeline.stages {
                        if let Some((_, id)) = pipeline.render_targets.get(&stage.render_target) {
                            if let Some(render_target) = self.render_targets.get(*id) {
                                let render_target_signature =
                                    MaterialRenderTargetSignature::new(render_target);
                                for (mesh_signature, middlewares) in &mesh_signatures_middlewares {
                                    self.cached_signatures.insert(MaterialSignature::new(
                                        mesh_signature.to_owned(),
                                        render_target_signature.to_owned(),
                                        stage.domain.to_owned(),
                                        middlewares.to_owned(),
                                    ));
                                }
                            }
                        }
                    }
                }
                let added = self
                    .cached_signatures
                    .difference(&old)
                    .collect::<HashSet<_>>();
                let removed = old
                    .difference(&self.cached_signatures)
                    .collect::<HashSet<_>>();
                for (id, material) in self.materials.iter_mut() {
                    if material.graph().is_some() {
                        for signature in &removed {
                            if let Err(error) = material.remove_version(context, signature) {
                                self.error_reporter.on_report(Error::Material(id, error));
                            }
                        }
                        for signature in &added {
                            let domain = if let Some(domain) = signature.domain() {
                                library.domain(domain)
                            } else {
                                None
                            };
                            let baked = material.graph().unwrap().bake(
                                signature,
                                domain,
                                library,
                                fragment_high_precision_support,
                            );
                            if let Ok(Some(baked)) = baked {
                                if let Err(error) =
                                    material.add_version(context, (*signature).to_owned(), baked)
                                {
                                    self.error_reporter.on_report(Error::Material(id, error));
                                }
                            }
                        }
                    }
                }
            } else {
                for (id, material) in self.materials.iter_mut() {
                    for signature in &old {
                        if let Err(error) = material.remove_version(context, signature) {
                            self.error_reporter.on_report(Error::Material(id, error));
                        }
                    }
                }
            }
        }
        for id in self.added_materials.drain() {
            if let Some(material) = self.materials.get_mut(id) {
                if material.graph().is_some() {
                    for signature in &self.cached_signatures {
                        let domain = if let Some(domain) = signature.domain() {
                            library.domain(domain)
                        } else {
                            None
                        };
                        let baked = material.graph().unwrap().bake(
                            signature,
                            domain,
                            library,
                            fragment_high_precision_support,
                        );
                        if let Ok(Some(baked)) = baked {
                            if let Err(error) =
                                material.add_version(context, (*signature).to_owned(), baked)
                            {
                                self.error_reporter.on_report(Error::Material(id, error));
                            }
                        }
                    }
                }
            }
        }
        self.dirty_signatures = false;
        self.added_materials.clear();
    }

    pub(crate) fn stage_resources(&self) -> RenderStageResources {
        RenderStageResources {
            render_targets: &self.render_targets,
            images: &self.images,
            virtual_images: &self.virtual_images,
            meshes: &self.meshes,
            virtual_meshes: &self.virtual_meshes,
            materials: &self.materials,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_sync() {
        fn foo<T>()
        where
            T: Send + Sync,
        {
            println!("{} is Send + Sync", std::any::type_name::<T>());
        }

        foo::<HaRenderer>();
    }
}
