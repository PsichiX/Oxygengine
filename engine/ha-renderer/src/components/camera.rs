use crate::{
    components::transform::HaTransform,
    ha_renderer::{HaRenderer, PipelineSource},
    material::common::MaterialRenderTargetSignature,
    math::*,
    pipeline::{render_queue::*, stage::*, *},
    render_target::RenderTargetViewport,
};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

#[derive(Ignite, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Ignite, Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Ignite, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
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
    pub width: usize,
    pub height: usize,
    pub transform_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

impl HaStageCameraInfo {
    /// (min, max)
    pub fn world_bounds(&self) -> (Vec3, Vec3) {
        let matrix = (self.projection_matrix * self.view_matrix).inverted();
        let vertices = [
            matrix.mul_point(Vec3::new(-1.0, -1.0, -1.0)),
            matrix.mul_point(Vec3::new(1.0, -1.0, -1.0)),
            matrix.mul_point(Vec3::new(1.0, 1.0, -1.0)),
            matrix.mul_point(Vec3::new(-1.0, 1.0, -1.0)),
            matrix.mul_point(Vec3::new(-1.0, -1.0, 1.0)),
            matrix.mul_point(Vec3::new(1.0, -1.0, 1.0)),
            matrix.mul_point(Vec3::new(1.0, 1.0, 1.0)),
            matrix.mul_point(Vec3::new(-1.0, 1.0, 1.0)),
        ];
        vertices
            .iter()
            .skip(1)
            .fold((vertices[0], vertices[0]), |(min, max), v| {
                (Vec3::partial_min(min, *v), Vec3::partial_max(max, *v))
            })
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaCamera {
    #[serde(default)]
    pub projection: HaCameraProjection,
    #[serde(default)]
    pub viewport: RenderTargetViewport,
    #[serde(default)]
    pub pipeline: PipelineSource,
    #[ignite(ignore)]
    #[serde(skip)]
    pub(crate) cached_pipeline: Option<PipelineId>,
}

impl HaCamera {
    pub fn pipeline_stage_info<'a, T: 'static>(
        &'a self,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = HaStageCameraInfo> + 'a> {
        let type_id = TypeId::of::<T>();
        let id = self.cached_pipeline?;
        let pipeline = renderer.pipelines.get(&id)?;
        Some(
            pipeline
                .stages
                .iter()
                .filter(move |stage| stage.type_id == type_id)
                .filter_map(|stage| {
                    let render_target = pipeline.render_targets.get(&stage.render_target)?;
                    let render_target = renderer.render_targets.get(render_target.1)?;
                    let width = render_target.width();
                    let height = render_target.height();
                    let (_, _, width, height) = self.viewport.rect(width, height);
                    let transform_matrix = camera_transform.world_matrix();
                    let view_matrix = transform_matrix.inverted();
                    let projection_matrix =
                        self.projection.matrix(Vec2::new(width as _, height as _));
                    Some(HaStageCameraInfo {
                        width,
                        height,
                        transform_matrix,
                        view_matrix,
                        projection_matrix,
                    })
                }),
        )
    }

    pub fn record_to_pipeline_stage<'a, T: 'static>(
        &'a self,
        renderer: &'a HaRenderer,
        camera_transform: &'a HaTransform,
    ) -> Option<impl Iterator<Item = (StageProcessInfo, Arc<RwLock<RenderQueue>>)> + 'a> {
        let type_id = TypeId::of::<T>();
        let id = self.cached_pipeline?;
        let pipeline = renderer.pipelines.get(&id)?;
        Some(
            pipeline
                .stages
                .iter()
                .filter(move |stage| stage.type_id == type_id)
                .filter_map(|stage| {
                    let render_target = pipeline.render_targets.get(&stage.render_target)?;
                    let render_target = renderer.render_targets.get(render_target.1)?;
                    let width = render_target.width();
                    let height = render_target.height();
                    let (x, y, width, height) = self.viewport.rect(width, height);
                    let transform_matrix = camera_transform.world_matrix();
                    let view_matrix = transform_matrix.inverted();
                    let projection_matrix =
                        self.projection.matrix(Vec2::new(width as _, height as _));
                    let info = StageProcessInfo {
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
                    if !matches!(self.viewport, RenderTargetViewport::Full) {
                        if let Ok(mut queue) = stage.render_queue.write() {
                            queue
                                .record(0, 0, RenderCommand::Viewport(x, y, width, height))
                                .ok()?;
                            queue.record(0, 0, RenderCommand::SortingBarrier).ok()?;
                        }
                    }
                    Some((info, Arc::clone(&stage.render_queue)))
                }),
        )
    }
}

impl Prefab for HaCamera {}

impl PrefabComponent for HaCamera {}
