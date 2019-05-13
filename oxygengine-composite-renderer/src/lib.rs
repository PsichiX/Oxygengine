extern crate oxygengine_core as core;

pub mod component;
pub mod composite_renderer;
pub mod math;
pub mod png_image_asset_protocol;
pub mod resource;
pub mod sprite_sheet_asset_protocol;
pub mod system;
pub mod tileset_asset_protocol;

pub mod prelude {
    pub use crate::{
        component::*, composite_renderer::*, math::*, png_image_asset_protocol::*, resource::*,
        sprite_sheet_asset_protocol::*, system::*, tileset_asset_protocol::*,
    };
}

use crate::{
    composite_renderer::CompositeRenderer,
    system::{
        CompositeRendererSystem, CompositeSpriteAnimationSystem, CompositeSpriteSheetSystem,
        CompositeSurfaceCacheSystem, CompositeTilemapAnimationSystem, CompositeTilemapSystem,
        CompositeTransformSystem,
    },
};
use core::{app::AppBuilder, assets::database::AssetsDatabase};

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
    builder.install_thread_local_system(CompositeSurfaceCacheSystem::<CR>::default());
    builder.install_thread_local_system(CompositeRendererSystem::<CR>::default());
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(png_image_asset_protocol::PngImageAssetProtocol);
    database.register(sprite_sheet_asset_protocol::SpriteSheetAssetProtocol);
    database.register(tileset_asset_protocol::TilesetAssetProtocol);
}
