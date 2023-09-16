extern crate oxygengine_animation as animation;
extern crate oxygengine_core as core;
extern crate oxygengine_utils as utils;

#[cfg(test)]
mod tests;

pub mod asset_protocols;
pub mod components;
pub mod constants;
pub mod ha_renderer;
pub mod image;
pub mod material;
pub mod math;
pub mod mesh;
pub mod pipeline;
pub mod platform;
pub mod render_target;
pub mod resources;
pub mod systems;

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    #[cfg(feature = "desktop")]
    pub use crate::platform::desktop::*;
    #[cfg(feature = "web")]
    pub use crate::platform::web::*;

    pub use crate::{
        asset_protocols::{
            atlas::*, font::*, image::*, material::*, mesh::*, rig::*, rig_animation::*,
            sprite_animation::*, tilemap::*, *,
        },
        builtin_material_function, builtin_material_functions, code_material_function,
        code_material_functions,
        components::{
            camera::*, gizmo::*, immediate_batch::*, material_instance::*, mesh_instance::*,
            postprocess::*, rig_instance::*, sprite_animation_instance::*, text_instance::*,
            tilemap_instance::*, transform::*, virtual_image_uniforms::*, visibility::*, volume::*,
            volume_overlap::*, volume_visibility::*, *,
        },
        constants::material_uniforms::*,
        graph_material_function,
        ha_renderer::*,
        image::*,
        material::{
            common::*,
            domains::{
                gizmo::*,
                screenspace::*,
                surface::{
                    circle::*, grid::*, immediate::*, quad::*, rig2d::*, text::*, tilemap::*,
                    triangles2d::*, *,
                },
                *,
            },
            graph::{function::*, node::*, *},
            *,
        },
        material_function, material_functions, material_graph, material_graph_input,
        material_graph_output, material_value_type,
        math::*,
        mesh::{
            controls::animation::*,
            geometry::*,
            rig::{deformer::*, skeleton::*, *},
            vertex_factory::*,
            *,
        },
        pipeline::{render_queue::*, stage::*, *},
        platform::*,
        render_target::*,
        resources::{camera_cache::*, gizmos::*, material_library::*, resource_mapping::*, *},
        rich_text,
        systems::{
            apply_sprite_animation_to_material::*, atlas::*, camera_cache::*, font::*,
            immediate_batch::*, mesh_bounds_gizmo::*, render_forward_stage::*,
            render_gizmo_stage::*, render_postprocess_stage::*, renderer::*, sprite_animation::*,
            tilemap::*, transform::*, virtual_image_uniforms::*, volume_overlap::*,
            volume_visibility::*, *,
        },
        Error, HaRendererBundleSetup, HasContextResources, ResourceReference, Resources,
    };
}

use crate::{
    asset_protocols::{
        atlas::AtlasAssetProtocol,
        font::FontAssetProtocol,
        image::{ImageAsset, ImageAssetProtocol},
        material::{MaterialAsset, MaterialAssetProtocol},
        mesh::{
            MeshAsset, MeshAssetProtocol, MeshVertexData, ScreenSpaceMeshAsset, SurfaceFactory,
            SurfaceMeshAsset,
        },
        rig::RigAssetProtocol,
        rig_animation::RigAnimationAssetProtocol,
        sprite_animation::SpriteAnimationAssetProtocol,
        tilemap::TileMapAssetProtocol,
    },
    components::{
        camera::{HaCamera, HaDefaultCamera},
        gizmo::HaGizmo,
        immediate_batch::HaImmediateBatch,
        material_instance::HaMaterialInstance,
        mesh_instance::HaMeshInstance,
        postprocess::HaPostProcess,
        rig_instance::HaRigInstance,
        sprite_animation_instance::HaSpriteAnimationInstance,
        text_instance::HaTextInstance,
        tilemap_instance::HaTileMapInstance,
        transform::HaTransform,
        virtual_image_uniforms::HaVirtualImageUniforms,
        visibility::HaVisibility,
        volume::HaVolume,
        volume_overlap::HaVolumeOverlap,
        volume_visibility::HaVolumeVisibility,
    },
    ha_renderer::HaRenderer,
    image::{ImageError, ImageId, ImageMode, ImageResourceMapping},
    material::{
        domains::{
            gizmo::{default_gizmo_color_material_graph, gizmo_domain_graph},
            screenspace::{
                default_screenspace_color_material_graph,
                default_screenspace_texture_material_graph, screenspace_domain_graph,
                ScreenSpaceQuadFactory,
            },
            surface::{
                default_surface_flat_color_material_graph, default_surface_flat_material_graph,
                default_surface_flat_sdf_text_material_graph,
                default_surface_flat_sdf_texture_2d_array_material_graph,
                default_surface_flat_sdf_texture_2d_material_graph,
                default_surface_flat_sdf_texture_3d_material_graph,
                default_surface_flat_text_material_graph,
                default_surface_flat_texture_2d_array_material_graph,
                default_surface_flat_texture_2d_material_graph,
                default_surface_flat_texture_3d_material_graph,
                default_surface_flat_virtual_uniform_texture_2d_array_material_graph,
                default_surface_flat_virtual_uniform_texture_2d_material_graph,
                default_surface_flat_virtual_uniform_texture_3d_material_graph,
                quad::SurfaceQuadFactory, surface_flat_domain_graph, SurfaceDomain,
            },
        },
        MaterialDrawOptions, MaterialError, MaterialId, MaterialResourceMapping,
    },
    mesh::{controls::animation::AnimationRigControl, MeshError, MeshId, MeshResourceMapping},
    render_target::{RenderTargetError, RenderTargetId},
    resources::{camera_cache::CameraCache, gizmos::Gizmos, material_library::MaterialLibrary},
    systems::{
        apply_sprite_animation_to_material::{
            ha_apply_sprite_animation_to_material, HaApplySpriteAnimationToMaterialSystemResources,
        },
        atlas::{ha_atlas_system, HaAtlasSystemCache, HaAtlasSystemResources},
        camera_cache::{ha_camera_cache_system, HaCameraCacheSystemResources},
        font::{ha_font_system, HaFontSystemCache, HaFontSystemResources},
        immediate_batch::{
            ha_immediate_batch_system, HaImmediateBatchSystemCache, HaImmediateBatchSystemResources,
        },
        mesh_bounds_gizmo::{ha_mesh_bounds_gizmo_system, HaMeshBoundsGizmoSystemResources},
        render_forward_stage::{
            ha_render_forward_stage_system, HaRenderForwardStageSystemResources,
        },
        render_gizmo_stage::{
            ha_render_gizmo_stage_system, HaRenderGizmoStageSystemCache,
            HaRenderGizmoStageSystemResources,
        },
        render_postprocess_stage::{
            ha_render_postprocess_stage_system, HaRenderPostProcessStageSystemCache,
            HaRenderPostProcessStageSystemResources,
        },
        renderer::{
            ha_renderer_execution_system, ha_renderer_maintenance_system,
            HaRendererExecutionSystemResources, HaRendererMaintenanceSystemCache,
            HaRendererMaintenanceSystemResources,
        },
        rig::{ha_rig_system, HaRigSystemCache, HaRigSystemResources},
        sprite_animation::{
            ha_sprite_animation, HaSpriteAnimationSystemCache, HaSpriteAnimationSystemResources,
        },
        tilemap::{ha_tilemap_system, HaTileMapSystemCache, HaTileMapSystemResources},
        transform::{ha_transform_system, HaTransformSystemResources},
        virtual_image_uniforms::{
            ha_virtual_image_uniforms, HaVirtualImageUniformsSystemResources,
        },
        volume_overlap::{
            ha_volume_overlap_system, HaVolumeOverlapSystemCache, HaVolumeOverlapSystemResources,
        },
        volume_visibility::{
            ha_volume_visibility_system, HaVolumeVisibilitySystemCache,
            HaVolumeVisibilitySystemResources,
        },
    },
};
use core::{
    app::AppBuilder,
    assets::{asset::Asset, database::AssetsDatabase},
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError, PipelineLayer},
        Component,
    },
    id::ID,
    prefab::PrefabManager,
    scripting::intuicio::core::{registry::Registry, struct_type::NativeStructBuilder},
};
use glow::HasContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait HasContextResources<T>
where
    T: HasContext,
{
    type Error;

    fn has_resources(&self) -> bool;
    fn context_initialize(&mut self, context: &T) -> Result<(), Self::Error>;
    fn context_release(&mut self, context: &T) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceReference<ID, VID = ID> {
    None,
    Asset(String),
    VirtualAsset(String),
    Id(ID),
    VirtualId { owner: VID, id: ID },
}

impl<ID, VID> Default for ResourceReference<ID, VID> {
    fn default() -> Self {
        Self::None
    }
}

impl<ID, VID> ResourceReference<ID, VID> {
    pub fn asset(&self) -> Option<&str> {
        match self {
            Self::Asset(asset) => Some(asset.as_str()),
            _ => None,
        }
    }

    pub fn virtual_asset(&self) -> Option<&str> {
        match self {
            Self::VirtualAsset(asset) => Some(asset.as_str()),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<&ID> {
        match self {
            Self::Id(id) => Some(id),
            _ => None,
        }
    }

    pub fn virtual_id(&self) -> Option<(&VID, &ID)> {
        match self {
            Self::VirtualId { owner, id } => Some((owner, id)),
            _ => None,
        }
    }
}

impl<ID, VID> ToString for ResourceReference<ID, VID>
where
    ID: std::fmt::Debug,
    VID: std::fmt::Debug,
{
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Resources<T> {
    cache: HashMap<ID<T>, T>,
    map: HashMap<ID<T>, String>,
    table: HashMap<String, ID<T>>,
}

impl<T> Default for Resources<T> {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            map: Default::default(),
            table: Default::default(),
        }
    }
}

impl<T> Resources<T> {
    pub fn add(&mut self, data: T) -> ID<T> {
        let id = ID::new();
        self.cache.insert(id, data);
        id
    }

    pub fn add_named(&mut self, name: String, data: T) -> ID<T> {
        let id = self.add(data);
        self.map.insert(id, name.to_owned());
        self.table.insert(name, id);
        id
    }

    pub fn remove(&mut self, id: ID<T>) -> Option<T> {
        if let Some(name) = self.map.remove(&id) {
            self.table.remove(&name);
        }
        self.cache.remove(&id)
    }

    pub fn remove_named(&mut self, name: &str) -> Option<T> {
        if let Some(id) = self.table.remove(name) {
            self.map.remove(&id);
            self.cache.remove(&id)
        } else {
            None
        }
    }

    pub fn id_by_name(&self, name: &str) -> Option<ID<T>> {
        self.table.get(name).copied()
    }

    pub fn get(&self, id: ID<T>) -> Option<&T> {
        self.cache.get(&id)
    }

    pub fn get_named(&self, name: &str) -> Option<&T> {
        if let Some(id) = self.table.get(name) {
            self.cache.get(id)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: ID<T>) -> Option<&mut T> {
        self.cache.get_mut(&id)
    }

    pub fn get_named_mut(&mut self, name: &str) -> Option<&mut T> {
        if let Some(id) = self.table.get(name) {
            self.cache.get_mut(id)
        } else {
            None
        }
    }

    pub fn with<F, R>(&mut self, id: ID<T>, mut f: F) -> Option<R>
    where
        F: FnMut(&T) -> R,
    {
        self.get_mut(id).map(|resource| f(resource))
    }

    pub fn with_named<F, R>(&mut self, name: &str, mut f: F) -> Option<R>
    where
        F: FnMut(&T) -> R,
    {
        self.get_named_mut(name).map(|resource| f(resource))
    }

    pub fn ids(&self) -> impl Iterator<Item = ID<T>> + '_ {
        self.cache.keys().copied()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> + '_ {
        self.table.keys().map(|k| k.as_str())
    }

    pub fn resources(&self) -> impl Iterator<Item = &T> + '_ {
        self.cache.values()
    }

    pub fn resources_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.cache.values_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (ID<T>, &T)> + '_ {
        self.cache.iter().map(|(id, resource)| (*id, resource))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ID<T>, &mut T)> + '_ {
        self.cache.iter_mut().map(|(id, resource)| (*id, resource))
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    RenderTarget(RenderTargetId, RenderTargetError),
    Mesh(MeshId, MeshError),
    Image(ImageId, ImageError),
    Material(MaterialId, MaterialError),
    Custom(String),
}

#[derive(Debug)]
pub struct HaRendererBundleSetup {
    renderer: HaRenderer,
    gizmos: Gizmos,
}

impl HaRendererBundleSetup {
    pub fn new(renderer: HaRenderer) -> Self {
        Self {
            renderer,
            gizmos: Default::default(),
        }
    }

    pub fn with_gizmos(mut self, gizmos: Gizmos) -> Self {
        self.gizmos = gizmos;
        self
    }
}

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    setup: HaRendererBundleSetup,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(setup.renderer);
    builder.install_resource(HaRendererMaintenanceSystemCache::default());
    builder.install_resource(HaAtlasSystemCache::default());
    builder.install_resource(HaFontSystemCache::default());
    builder.install_resource(HaTileMapSystemCache::default());
    builder.install_resource(HaSpriteAnimationSystemCache::default());
    builder.install_resource(HaRigSystemCache::default());
    builder.install_resource(HaVolumeVisibilitySystemCache::default());
    builder.install_resource(HaVolumeOverlapSystemCache::default());
    builder.install_resource(HaRenderGizmoStageSystemCache::default());
    builder.install_resource(HaRenderPostProcessStageSystemCache::default());
    builder.install_resource(HaImmediateBatchSystemCache::default());
    builder.install_resource(MaterialLibrary::default());
    builder.install_resource(ImageResourceMapping::default());
    builder.install_resource(MeshResourceMapping::default());
    builder.install_resource(MaterialResourceMapping::default());
    builder.install_resource(CameraCache::default());
    builder.install_resource(setup.gizmos);

    // NOTE: ORDER MATTERS! transform first, renderer second, then the others - dependencies always first.
    builder.install_system_on_layer::<HaTransformSystemResources>(
        "transform",
        ha_transform_system,
        &[],
        PipelineLayer::Pre,
        false,
    )?;
    builder.install_system_on_layer::<HaRendererMaintenanceSystemResources>(
        "renderer-maintenance",
        ha_renderer_maintenance_system,
        &[],
        PipelineLayer::Main,
        true,
    )?;
    builder.install_system_on_layer::<HaRendererExecutionSystemResources>(
        "renderer-execution",
        ha_renderer_execution_system,
        &[],
        PipelineLayer::Post,
        true,
    )?;
    builder.install_system::<HaCameraCacheSystemResources>(
        "camera-cache",
        ha_camera_cache_system,
        &[],
    )?;
    builder.install_system::<HaRenderForwardStageSystemResources>(
        "renderer-forward-stage",
        ha_render_forward_stage_system,
        &[],
    )?;
    builder.install_system::<HaRenderPostProcessStageSystemResources>(
        "renderer-postprocess-stage",
        ha_render_postprocess_stage_system,
        &[],
    )?;
    builder.install_system::<HaRenderGizmoStageSystemResources>(
        "renderer-gizmo-stage",
        ha_render_gizmo_stage_system,
        &[],
    )?;
    builder.install_system::<HaAtlasSystemResources>("atlas", ha_atlas_system, &[])?;
    builder.install_system::<HaFontSystemResources>("font", ha_font_system, &[])?;
    builder.install_system::<HaTileMapSystemResources>("tilemap", ha_tilemap_system, &[])?;
    builder.install_system::<HaRigSystemResources>("rig", ha_rig_system, &[])?;
    builder.install_system::<HaSpriteAnimationSystemResources>(
        "sprite-animation",
        ha_sprite_animation,
        &[],
    )?;
    builder.install_system::<HaApplySpriteAnimationToMaterialSystemResources>(
        "apply-sprite-animation-to-material",
        ha_apply_sprite_animation_to_material,
        &["sprite-animation"],
    )?;
    builder.install_system::<HaVirtualImageUniformsSystemResources>(
        "virtual-image-uniforms",
        ha_virtual_image_uniforms,
        &["apply-sprite-animation-to-material"],
    )?;
    builder.install_system::<HaVolumeVisibilitySystemResources>(
        "volume-visibility",
        ha_volume_visibility_system,
        &[],
    )?;
    builder.install_system::<HaVolumeOverlapSystemResources>(
        "volume-overlap",
        ha_volume_overlap_system,
        &[],
    )?;
    builder.install_system::<HaMeshBoundsGizmoSystemResources>(
        "mesh-bounds-gizmo",
        ha_mesh_bounds_gizmo_system,
        &[],
    )?;

    Ok(())
}

pub fn immediate_batch_system_installer<PB, C>(
    builder: &mut AppBuilder<PB>,
    postfix: &str,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    C: Component + SurfaceDomain + Default + Copy + Send + Sync,
{
    builder.install_system::<HaImmediateBatchSystemResources<C>>(
        &format!("immediate-batch-system-{}", postfix),
        ha_immediate_batch_system::<C>,
        &[],
    )?;
    Ok(())
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(AtlasAssetProtocol);
    database.register(FontAssetProtocol);
    database.register(ImageAssetProtocol);
    database.register(MaterialAssetProtocol);
    database.register(MeshAssetProtocol);
    database.register(SpriteAnimationAssetProtocol);
    database.register(RigAnimationAssetProtocol);
    database.register(RigAssetProtocol);
    database.register(TileMapAssetProtocol);

    database.insert(Asset::new(
        "mesh",
        "@mesh/surface/quad/p",
        MeshAsset::Surface(SurfaceMeshAsset {
            vertex_data: MeshVertexData {
                texture: false,
                color: false,
                skinning: false,
                deforming: false,
            },
            factory: SurfaceFactory::Quad(SurfaceQuadFactory::default()),
        }),
    ));
    database.insert(Asset::new(
        "mesh",
        "@mesh/surface/quad/pt",
        MeshAsset::Surface(SurfaceMeshAsset {
            vertex_data: MeshVertexData {
                texture: true,
                color: false,
                skinning: false,
                deforming: false,
            },
            factory: SurfaceFactory::Quad(SurfaceQuadFactory::default()),
        }),
    ));
    database.insert(Asset::new(
        "mesh",
        "@mesh/surface/quad/pc",
        MeshAsset::Surface(SurfaceMeshAsset {
            vertex_data: MeshVertexData {
                texture: false,
                color: true,
                skinning: false,
                deforming: false,
            },
            factory: SurfaceFactory::Quad(SurfaceQuadFactory::default()),
        }),
    ));
    database.insert(Asset::new(
        "mesh",
        "@mesh/surface/quad/ptc",
        MeshAsset::Surface(SurfaceMeshAsset {
            vertex_data: MeshVertexData {
                texture: true,
                color: true,
                skinning: false,
                deforming: false,
            },
            factory: SurfaceFactory::Quad(SurfaceQuadFactory::default()),
        }),
    ));
    database.insert(Asset::new(
        "mesh",
        "@mesh/screenspace",
        MeshAsset::ScreenSpace(ScreenSpaceMeshAsset(ScreenSpaceQuadFactory)),
    ));

    database.insert(Asset::new(
        "image",
        "@image/empty-2d",
        ImageAsset::color([0, 0, 0, 0], ImageMode::Image2d),
    ));
    database.insert(Asset::new(
        "image",
        "@image/white-2d",
        ImageAsset::color([255, 255, 255, 255], ImageMode::Image2d),
    ));
    database.insert(Asset::new(
        "image",
        "@image/error-2d",
        ImageAsset::color([255, 0, 255, 255], ImageMode::Image2d),
    ));
    database.insert(Asset::new(
        "image",
        "@image/checkerboard-2d",
        ImageAsset::bytes(
            2,
            2,
            vec![
                0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
            ],
            ImageMode::Image2d,
        ),
    ));
    database.insert(Asset::new(
        "image",
        "@image/test-2d",
        ImageAsset::bytes(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 127, 127, 127, 255,
            ],
            ImageMode::Image2d,
        ),
    ));
    database.insert(Asset::new(
        "image",
        "@image/empty-2d-array",
        ImageAsset::color([0, 0, 0, 0], ImageMode::Image2dArray),
    ));
    database.insert(Asset::new(
        "image",
        "@image/white-2d-array",
        ImageAsset::color([255, 255, 255, 255], ImageMode::Image2dArray),
    ));
    database.insert(Asset::new(
        "image",
        "@image/error-2d-array",
        ImageAsset::color([255, 0, 255, 255], ImageMode::Image2dArray),
    ));
    database.insert(Asset::new(
        "image",
        "@image/checkerboard-2d-array",
        ImageAsset::bytes(
            2,
            2,
            vec![
                0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
            ],
            ImageMode::Image2dArray,
        ),
    ));
    database.insert(Asset::new(
        "image",
        "@image/test-2d-array",
        ImageAsset::bytes(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 127, 127, 127, 255,
            ],
            ImageMode::Image2dArray,
        ),
    ));

    database.insert(Asset::new(
        "material",
        "@material/domain/surface/flat",
        MaterialAsset::Domain(surface_flat_domain_graph()),
    ));
    database.insert(Asset::new(
        "material",
        "@material/domain/screenspace",
        MaterialAsset::Domain(screenspace_domain_graph()),
    ));
    database.insert(Asset::new(
        "material",
        "@material/domain/gizmo",
        MaterialAsset::Domain(gizmo_domain_graph()),
    ));

    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::default(),
            content: default_surface_flat_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/color",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_color_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/texture-2d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_texture_2d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/texture-2d-array",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_texture_2d_array_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/texture-3d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_texture_3d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/sdf-texture-2d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_sdf_texture_2d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/sdf-texture-2d-array",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_sdf_texture_2d_array_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/sdf-texture-3d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_sdf_texture_3d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/virtual-uniform-texture-2d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_virtual_uniform_texture_2d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/virtual-uniform-texture-2d-array",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_virtual_uniform_texture_2d_array_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/virtual-uniform-texture-3d",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_virtual_uniform_texture_3d_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/text",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_text_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/surface/flat/sdf-text",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: MaterialDrawOptions::transparent(),
            content: default_surface_flat_sdf_text_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/screenspace/color",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: Default::default(),
            content: default_screenspace_color_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/screenspace/texture",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: Default::default(),
            content: default_screenspace_texture_material_graph(),
        },
    ));
    database.insert(Asset::new(
        "material",
        "@material/graph/gizmo/color",
        MaterialAsset::Graph {
            default_values: Default::default(),
            draw_options: Default::default(),
            content: default_gizmo_color_material_graph(),
        },
    ));
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<HaCamera>("HaCamera");
    prefabs.register_component_factory::<HaDefaultCamera>("HaDefaultCamera");
    prefabs.register_component_factory::<HaMaterialInstance>("HaMaterialInstance");
    prefabs.register_component_factory::<HaMeshInstance>("HaMeshInstance");
    prefabs.register_component_factory::<HaSpriteAnimationInstance>("HaSpriteAnimationInstance");
    prefabs.register_component_factory::<HaRigInstance>("HaRigInstance");
    prefabs.register_component_factory::<HaTextInstance>("HaTextInstance");
    prefabs.register_component_factory::<HaTileMapInstance>("HaTileMapInstance");
    prefabs.register_component_factory::<HaTransform>("HaTransform");
    prefabs.register_component_factory::<HaVirtualImageUniforms>("HaVirtualImageUniforms");
    prefabs.register_component_factory::<HaVisibility>("HaVisibility");
    prefabs.register_component_factory::<HaVolume>("HaVolume");
    prefabs.register_component_factory::<HaVolumeVisibility>("HaVolumeVisibility");
    prefabs.register_component_factory::<HaVolumeOverlap>("HaVolumeOverlap");
    prefabs.register_component_factory::<HaGizmo>("HaGizmo");
    prefabs.register_component_factory::<HaPostProcess>("HaPostProcess");
}

pub fn immediate_batch_prefab_installer<C>(postfix: &str, prefabs: &mut PrefabManager)
where
    C: Component + SurfaceDomain + Default + Copy + Send + Sync,
{
    prefabs.register_component_factory::<HaImmediateBatch<C>>(&format!(
        "HaImmediateBatch-{}",
        postfix
    ));
}

pub fn scripting_installer(registry: &mut Registry) {
    registry.add_struct(
        NativeStructBuilder::new_named::<HaRigInstance>("HaRigInstance")
            .module_name("ha_renderer")
            .build(),
    );
    AnimationRigControl::install(registry);
}
