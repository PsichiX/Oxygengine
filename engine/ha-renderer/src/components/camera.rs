use crate::{
    components::transform::HaTransform,
    ha_renderer::{HaRenderer, PipelineSource},
    material::common::MaterialRenderTargetSignature,
    math::*,
    pipeline::{render_queue::*, stage::*, *},
    render_target::RenderTargetClipArea,
};
use core::{
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HaCameraOrtographicScaling {
    None,
    Stretch(Vec2),
    FitHorizontal(Scalar),
    FitVertical(Scalar),
    /// (view size, fit inside)
    FitToView(Vec2, bool),
}

impl Default for HaCameraOrtographicScaling {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HaCameraOrthographic {
    #[serde(default)]
    pub scaling: HaCameraOrtographicScaling,
    #[serde(default)]
    pub centered: bool,
    #[serde(default)]
    pub ignore_depth_planes: bool,
}

impl HaCameraOrthographic {
    pub fn matrix(&self, size: Vec2) -> Mat4 {
        let size = match self.scaling {
            HaCameraOrtographicScaling::None => size,
            HaCameraOrtographicScaling::Stretch(size) => size,
            HaCameraOrtographicScaling::FitHorizontal(width) => {
                let height = width * size.y / size.x;
                Vec2::new(width, height)
            }
            HaCameraOrtographicScaling::FitVertical(height) => {
                let width = height * size.x / size.y;
                Vec2::new(width, height)
            }
            HaCameraOrtographicScaling::FitToView(view, inside) => {
                let aspect = size.x / size.y;
                let view_aspect = view.x / view.y;
                let (width, height) = if (aspect >= view_aspect) != inside {
                    (size.x * view.x / size.y, view.y)
                } else {
                    (view.x, size.y * view.y / size.x)
                };
                Vec2::new(width, height)
            }
        };
        let frustum = if self.centered {
            let half_size = size * 0.5;
            FrustumPlanes {
                left: -half_size.x,
                right: half_size.x,
                top: -half_size.y,
                bottom: half_size.y,
                near: -1.0,
                far: 1.0,
            }
        } else {
            FrustumPlanes {
                left: 0.0,
                right: size.x,
                top: 0.0,
                bottom: size.y,
                near: -1.0,
                far: 1.0,
            }
        };
        if self.ignore_depth_planes {
            Mat4::orthographic_without_depth_planes(frustum)
        } else {
            Mat4::orthographic_rh_zo(frustum)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HaCameraPerspective {
    #[serde(default = "HaCameraPerspective::default_fov")]
    pub fov: Scalar,
    #[serde(default = "HaCameraPerspective::default_near")]
    pub near: Scalar,
    #[serde(default)]
    pub far: Option<Scalar>,
}

impl Default for HaCameraPerspective {
    fn default() -> Self {
        Self {
            fov: Self::default_fov(),
            near: Self::default_near(),
            far: None,
        }
    }
}

impl HaCameraPerspective {
    fn default_fov() -> Scalar {
        #[cfg(feature = "scalar64")]
        {
            50.0_f64.to_radians()
        }
        #[cfg(not(feature = "scalar64"))]
        {
            50.0_f32.to_radians()
        }
    }

    fn default_near() -> Scalar {
        1.0
    }

    pub fn matrix(&self, size: Vec2) -> Mat4 {
        if let Some(far) = self.far {
            Mat4::perspective_fov_rh_zo(self.fov, size.x, size.y, self.near, far)
        } else {
            Mat4::infinite_perspective_rh(self.fov, size.x / size.y, self.near)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HaCameraProjection {
    Orthographic(#[serde(default)] HaCameraOrthographic),
    Perspective(#[serde(default)] HaCameraPerspective),
}

impl Default for HaCameraProjection {
    fn default() -> Self {
        Self::Orthographic(Default::default())
    }
}

impl HaCameraProjection {
    pub fn matrix(&self, size: Vec2) -> Mat4 {
        match self {
            Self::Orthographic(orthographic) => orthographic.matrix(size),
            Self::Perspective(perspective) => perspective.matrix(size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HaStageCameraInfo {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub transform_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

impl HaStageCameraInfo {
    pub fn render_target_to_screen(&self, mut point: Vec2) -> Vec2 {
        point.x = (2.0 * (point.x - self.x as Scalar) / self.width as Scalar) - 1.0;
        point.y = (-2.0 * (point.y - self.y as Scalar) / self.height as Scalar) + 1.0;
        point
    }

    pub fn screen_to_render_target(&self, mut point: Vec2) -> Vec2 {
        point.x = (point.x + 1.0) * 0.5 * self.width as Scalar + self.x as Scalar;
        point.y = (point.y + 1.0) * 0.5 * self.height as Scalar + self.y as Scalar;
        point
    }

    pub fn world_to_screen(&self) -> Mat4 {
        self.projection_matrix * self.view_matrix
    }

    pub fn world_to_screen_point(&self, point: Vec3) -> Vec3 {
        self.world_to_screen().mul_point(point)
    }

    pub fn world_to_screen_direction(&self, point: Vec3) -> Vec3 {
        self.world_to_screen().mul_direction(point)
    }

    pub fn screen_to_world(&self) -> Mat4 {
        self.world_to_screen().inverted()
    }

    pub fn screen_to_world_point(&self, point: Vec3) -> Vec3 {
        self.screen_to_world().mul_point(point)
    }

    pub fn screen_to_world_direction(&self, point: Vec3) -> Vec3 {
        self.screen_to_world().mul_direction(point)
    }

    /// [(plane center, plane outward normal); 6]
    pub fn world_planes(&self) -> [(Vec3, Vec3); 6] {
        let matrix = self.screen_to_world();
        [
            (
                matrix.mul_point(Vec3::new(-1.0, 0.0, 0.0)),
                matrix.mul_direction(Vec3::new(-1.0, 0.0, 0.0)),
            ),
            (
                matrix.mul_point(Vec3::new(1.0, 0.0, 0.0)),
                matrix.mul_direction(Vec3::new(1.0, 0.0, 0.0)),
            ),
            (
                matrix.mul_point(Vec3::new(0.0, -1.0, 0.0)),
                matrix.mul_direction(Vec3::new(0.0, -1.0, 0.0)),
            ),
            (
                matrix.mul_point(Vec3::new(0.0, 1.0, 0.0)),
                matrix.mul_direction(Vec3::new(0.0, 1.0, 0.0)),
            ),
            (
                matrix.mul_point(Vec3::new(0.0, 0.0, -1.0)),
                matrix.mul_direction(Vec3::new(0.0, 0.0, -1.0)),
            ),
            (
                matrix.mul_point(Vec3::new(0.0, 0.0, 1.0)),
                matrix.mul_direction(Vec3::new(0.0, 0.0, 1.0)),
            ),
        ]
    }

    pub fn is_inside_world_bounds(&self, point: Vec3) -> bool {
        let point = self.world_to_screen_point(point);
        point.x >= -1.0
            && point.x <= 1.0
            && point.y >= -1.0
            && point.y <= 1.0
            && point.z >= -1.0
            && point.z <= 1.0
    }

    pub fn world_vertices(&self) -> [Vec3; 8] {
        let matrix = self.screen_to_world();
        [
            matrix.mul_point(Vec3::new(-1.0, -1.0, -1.0)),
            matrix.mul_point(Vec3::new(1.0, -1.0, -1.0)),
            matrix.mul_point(Vec3::new(1.0, 1.0, -1.0)),
            matrix.mul_point(Vec3::new(-1.0, 1.0, -1.0)),
            matrix.mul_point(Vec3::new(-1.0, -1.0, 1.0)),
            matrix.mul_point(Vec3::new(1.0, -1.0, 1.0)),
            matrix.mul_point(Vec3::new(1.0, 1.0, 1.0)),
            matrix.mul_point(Vec3::new(-1.0, 1.0, 1.0)),
        ]
    }

    /// (min, max)
    pub fn world_bounds(&self) -> (Vec3, Vec3) {
        let vertices = self.world_vertices();
        vertices
            .iter()
            .skip(1)
            .fold((vertices[0], vertices[0]), |(min, max), v| {
                (Vec3::partial_min(min, *v), Vec3::partial_max(max, *v))
            })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaCamera {
    #[serde(default)]
    pub projection: HaCameraProjection,
    #[serde(default)]
    pub clip_area: RenderTargetClipArea,
    #[serde(default)]
    pub pipeline: PipelineSource,
    #[serde(skip)]
    pub(crate) cached_pipeline: Option<PipelineId>,
}

impl HaCamera {
    pub fn with_projection(mut self, projection: HaCameraProjection) -> Self {
        self.projection = projection;
        self
    }

    pub fn with_clip_area(mut self, clip_area: RenderTargetClipArea) -> Self {
        self.clip_area = clip_area;
        self
    }

    pub fn with_pipeline(mut self, pipeline: PipelineSource) -> Self {
        self.pipeline = pipeline;
        self
    }

    pub fn pipeline_stage_info<'a, T: 'static>(
        &'a self,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = HaStageCameraInfo> + 'a> {
        Some(
            self.pipeline_stage_info_raw(Some(TypeId::of::<T>()), renderer, camera_transform)?
                .map(|(_, info)| info),
        )
    }

    pub fn pipeline_stage_info_raw<'a>(
        &'a self,
        type_id: Option<TypeId>,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = (TypeId, HaStageCameraInfo)> + 'a> {
        let id = self.cached_pipeline?;
        let pipeline = renderer.pipelines.get(&id)?;
        Some(
            pipeline
                .stages
                .iter()
                .filter(move |stage| type_id.map(|tid| stage.type_id == tid).unwrap_or(true))
                .filter_map(|stage| {
                    let render_target = pipeline.render_targets.get(&stage.render_target)?;
                    let render_target = renderer.render_targets.get(render_target.1)?;
                    let width = render_target.width();
                    let height = render_target.height();
                    let (x, y, width, height) = self.clip_area.rect(width, height);
                    let transform_matrix = camera_transform.world_matrix();
                    let view_matrix = transform_matrix.inverted();
                    let projection_matrix =
                        self.projection.matrix(Vec2::new(width as _, height as _));
                    Some((
                        stage.type_id,
                        HaStageCameraInfo {
                            x,
                            y,
                            width,
                            height,
                            transform_matrix,
                            view_matrix,
                            projection_matrix,
                        },
                    ))
                }),
        )
    }

    pub fn record_to_pipeline_stage<'a, T: 'static>(
        &'a self,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = (StageProcessInfo, Arc<RwLock<RenderQueue>>)> + 'a> {
        Some(
            self.record_to_pipeline_stage_raw(Some(TypeId::of::<T>()), renderer, camera_transform)?
                .map(|(_, info, queue)| (info, queue)),
        )
    }

    pub fn record_to_pipeline_stage_raw<'a>(
        &'a self,
        type_id: Option<TypeId>,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = (TypeId, StageProcessInfo, Arc<RwLock<RenderQueue>>)> + 'a>
    {
        let id = self.cached_pipeline?;
        let pipeline = renderer.pipelines.get(&id)?;
        Some(
            pipeline
                .stages
                .iter()
                .filter(move |stage| type_id.map(|tid| stage.type_id == tid).unwrap_or(true))
                .filter_map(|stage| {
                    let render_target = pipeline.render_targets.get(&stage.render_target)?;
                    let render_target = renderer.render_targets.get(render_target.1)?;
                    let target_width = render_target.width();
                    let target_height = render_target.height();
                    let (x, y, width, height) = self.clip_area.rect(target_width, target_height);
                    let transform_matrix = camera_transform.world_matrix();
                    let view_matrix = transform_matrix.inverted();
                    let projection_matrix =
                        self.projection.matrix(Vec2::new(width as _, height as _));
                    let info = StageProcessInfo {
                        x,
                        y,
                        width,
                        height,
                        transform_matrix,
                        view_matrix,
                        projection_matrix,
                        material_render_target_signature: MaterialRenderTargetSignature::new(
                            render_target,
                        ),
                        domain: stage.domain.to_owned(),
                        filters: stage.filters.to_owned(),
                    };
                    if let Ok(mut queue) = stage.render_queue.write() {
                        let mut queue = queue.auto_recorder(None);
                        queue
                            .record(RenderCommand::Viewport(x, y, width, height))
                            .ok()?;
                        queue
                            .record(RenderCommand::PushScissor(x, y, width, height, true))
                            .ok()?;
                        queue.record(RenderCommand::SortingBarrier).ok()?;
                    }
                    Some((stage.type_id, info, Arc::clone(&stage.render_queue)))
                }),
        )
    }
}

impl Prefab for HaCamera {}
impl PrefabComponent for HaCamera {}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct HaDefaultCamera;

impl Prefab for HaDefaultCamera {}
impl PrefabComponent for HaDefaultCamera {}
