extern crate oxygengine_core as core;
extern crate oxygengine_utils as utils;

pub mod component;
pub mod composite_renderer;
pub mod font_asset_protocol;
pub mod font_face_asset_protocol;
pub mod map_asset_protocol;
pub mod math;
pub mod mesh_asset_protocol;
pub mod png_image_asset_protocol;
pub mod resource;
pub mod sprite_sheet_asset_protocol;
pub mod system;
pub mod tileset_asset_protocol;
pub mod ui_theme_asset_protocol;

pub mod prelude {
    pub use crate::{
        component::*, composite_renderer::*, font_asset_protocol::*, font_face_asset_protocol::*,
        map_asset_protocol::*, math::*, mesh_asset_protocol::*, png_image_asset_protocol::*,
        resource::*, sprite_sheet_asset_protocol::*, system::*, tileset_asset_protocol::*,
        ui_theme_asset_protocol::*,
    };
}

use crate::{
    component::*,
    composite_renderer::CompositeRenderer,
    system::{
        CompositeCameraCacheSystem, CompositeMapSystem, CompositeMeshSystem,
        CompositeRendererSystem, CompositeSpriteAnimationSystem, CompositeSpriteSheetSystem,
        CompositeSurfaceCacheSystem, CompositeTilemapAnimationSystem, CompositeTilemapSystem,
        CompositeTransformSystem, CompositeUiSystem,
    },
};
use core::{
    app::AppBuilder, assets::database::AssetsDatabase, ignite_proxy, prefab::PrefabManager,
};

ignite_proxy! {
    struct Grid2d<T> {}
}

pub fn bundle_installer<'a, 'b, CR>(builder: &mut AppBuilder<'a, 'b>, data: CR)
where
    CR: CompositeRenderer + 'static,
{
    builder.install_resource(data);
    builder.install_system(CompositeTransformSystem, "transform", &[]);
    builder.install_system(CompositeSpriteAnimationSystem, "sprite_animation", &[]);
    builder.install_system(CompositeTilemapAnimationSystem, "tilemap_animation", &[]);
    builder.install_system(
        CompositeSpriteSheetSystem::default(),
        "sprite_sheet",
        &["sprite_animation"],
    );
    builder.install_system(
        CompositeTilemapSystem::default(),
        "tilemap",
        &["tilemap_animation"],
    );
    builder.install_system(CompositeMapSystem::default(), "map", &[]);
    builder.install_system(CompositeMeshSystem::default(), "mesh", &[]);
    builder.install_thread_local_system(CompositeUiSystem::<CR>::default());
    builder.install_thread_local_system(CompositeCameraCacheSystem::<CR>::default());
    builder.install_thread_local_system(CompositeSurfaceCacheSystem::<CR>::default());
    builder.install_thread_local_system(CompositeRendererSystem::<CR>::default());
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(png_image_asset_protocol::PngImageAssetProtocol);
    database.register(sprite_sheet_asset_protocol::SpriteSheetAssetProtocol);
    database.register(tileset_asset_protocol::TilesetAssetProtocol);
    database.register(map_asset_protocol::MapAssetProtocol);
    database.register(ui_theme_asset_protocol::UiThemeAssetProtocol);
    database.register(font_asset_protocol::FontAssetProtocol);
    database.register(font_face_asset_protocol::FontFaceAssetProtocol);
    database.register(mesh_asset_protocol::MeshAssetProtocol);
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
    prefabs.register_component_factory::<CompositeUiElement>("CompositeUiElement");
}
