extern crate oxygengine_animation as anims;
extern crate oxygengine_core as core;
extern crate oxygengine_utils as utils;

pub mod component;
pub mod composite_renderer;
pub mod font_asset_protocol;
pub mod font_face_asset_protocol;
pub mod jpg_image_asset_protocol;
pub mod map_asset_protocol;
pub mod math;
pub mod mesh_animation_asset_protocol;
pub mod mesh_asset_protocol;
pub mod png_image_asset_protocol;
pub mod resource;
pub mod sprite_sheet_asset_protocol;
pub mod svg_image_asset_protocol;
pub mod system;
pub mod tileset_asset_protocol;

pub mod prelude {
    pub use crate::{
        component::*,
        composite_renderer::*,
        font_asset_protocol::*,
        font_face_asset_protocol::*,
        jpg_image_asset_protocol::*,
        map_asset_protocol::*,
        math::*,
        mesh_animation_asset_protocol::*,
        mesh_asset_protocol::*,
        png_image_asset_protocol::*,
        resource::*,
        sprite_sheet_asset_protocol::*,
        svg_image_asset_protocol::*,
        system::{
            camera_cache::*, map::*, mesh::*, mesh_animation::*, renderer::*, sprite_animation::*,
            sprite_sheet::*, surface_cache::*, tilemap::*, tilemap_animation::*, transform::*, *,
        },
        tileset_asset_protocol::*,
    };
}

use crate::{
    component::*,
    composite_renderer::CompositeRenderer,
    resource::*,
    system::{
        camera_cache::*, map::*, mesh::*, mesh_animation::*, renderer::*, sprite_animation::*,
        sprite_sheet::*, surface_cache::*, tilemap::*, tilemap_animation::*, transform::*,
    },
};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    ignite_proxy,
    prefab::PrefabManager,
};

ignite_proxy! {
    struct Grid2d<T> {}
}

pub fn bundle_installer<PB, CR>(
    builder: &mut AppBuilder<PB>,
    data: CR,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    CR: CompositeRenderer + 'static,
{
    builder.install_resource(data);
    builder.install_resource(CompositeTransformCache::default());
    builder.install_resource(CompositeCameraCache::default());
    builder.install_resource(CompositeMeshAnimationSystemCache::default());
    builder.install_resource(CompositeSpriteSheetSystemCache::default());
    builder.install_resource(CompositeTilemapSystemCache::default());
    builder.install_resource(CompositeMeshSystemCache::default());
    builder.install_resource(CompositeMapSystemCache::default());
    builder.install_resource(CompositeSurfaceCacheSystemCache::default());

    builder.install_system::<CompositeTransformSystemResources>(
        "transform",
        composite_transform_system,
        &[],
    )?;
    builder.install_system::<CompositeSpriteAnimationSystemResources>(
        "sprite-animation",
        composite_sprite_animation_system,
        &[],
    )?;
    builder.install_system::<CompositeTilemapAnimationSystemResources>(
        "tilemap-animation",
        composite_tilemap_animation_system,
        &[],
    )?;
    builder.install_system::<CompositeMeshAnimationSystemResources>(
        "mesh-animation",
        composite_mesh_animation_system,
        &[],
    )?;
    builder.install_system::<CompositeSpriteSheetSystemResources>(
        "sprite-sheet",
        composite_sprite_sheet_system,
        &["sprite-animation"],
    )?;
    builder.install_system::<CompositeTilemapSystemResources>(
        "tilemap",
        composite_tilemap_system,
        &["tilemap-animation"],
    )?;
    builder.install_system::<CompositeMeshSystemResources>(
        "mesh",
        composite_mesh_system,
        &["mesh-animation"],
    )?;
    builder.install_system::<CompositeMapSystemResources>("map", composite_map_system, &[])?;
    builder.install_system::<CompositeCameraCacheSystemResources<CR>>(
        "camera-cache",
        composite_camera_cache_system::<CR>,
        &[],
    )?;
    builder.install_system::<CompositeSurfaceCacheSystemResources<CR>>(
        "surface-cache",
        composite_surface_cache_system::<CR>,
        &[],
    )?;
    builder.install_system::<CompositeRendererSystemResources<CR>>(
        "renderer",
        composite_renderer_system::<CR>,
        &[],
    )?;

    Ok(())
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(png_image_asset_protocol::PngImageAssetProtocol);
    database.register(jpg_image_asset_protocol::JpgImageAssetProtocol);
    database.register(svg_image_asset_protocol::SvgImageAssetProtocol);
    database.register(sprite_sheet_asset_protocol::SpriteSheetAssetProtocol);
    database.register(tileset_asset_protocol::TilesetAssetProtocol);
    database.register(map_asset_protocol::MapAssetProtocol);
    database.register(font_asset_protocol::FontAssetProtocol);
    database.register(font_face_asset_protocol::FontFaceAssetProtocol);
    database.register(mesh_asset_protocol::MeshAssetProtocol);
    database.register(mesh_animation_asset_protocol::MeshAnimationAssetProtocol);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<CompositeVisibility>("CompositeVisibility");
    prefabs.register_component_factory::<CompositeSurfaceCache>("CompositeSurfaceCache");
    prefabs.register_component_factory::<CompositeRenderable>("CompositeRenderable");
    prefabs.register_component_factory::<CompositeRenderableStroke>("CompositeRenderableStroke");
    prefabs.register_component_factory::<CompositeTransform>("CompositeTransform");
    prefabs.register_component_factory::<CompositeRenderLayer>("CompositeRenderLayer");
    prefabs.register_component_factory::<CompositeRenderDepth>("CompositeRenderDepth");
    prefabs.register_component_factory::<CompositeRenderAlpha>("CompositeRenderAlpha");
    prefabs.register_component_factory::<CompositeCameraAlignment>("CompositeCameraAlignment");
    prefabs.register_component_factory::<CompositeEffect>("CompositeEffect");
    prefabs.register_component_factory::<CompositeCamera>("CompositeCamera");
    prefabs.register_component_factory::<CompositeSprite>("CompositeSprite");
    prefabs.register_component_factory::<CompositeSpriteAnimation>("CompositeSpriteAnimation");
    prefabs.register_component_factory::<CompositeTilemap>("CompositeTilemap");
    prefabs.register_component_factory::<CompositeTilemapAnimation>("CompositeTilemapAnimation");
    prefabs.register_component_factory::<CompositeMapChunk>("CompositeMapChunk");
    prefabs.register_component_factory::<CompositeMesh>("CompositeMesh");
    prefabs.register_component_factory::<CompositeMeshAnimation>("CompositeMeshAnimation");
}
