extern crate oxygengine_core as core;
extern crate png;

pub mod component;
pub mod composite_renderer;
pub mod math;
pub mod png_image_asset_protocol;
pub mod resource;
pub mod system;

use crate::{
    composite_renderer::CompositeRenderer,
    system::{CompositeRendererSystem, CompositeTransformSystem},
};
use core::{app::AppBuilder, assets::database::AssetsDatabase};

pub fn bundle_installer<'a, 'b, CR: 'static>(builder: &mut AppBuilder<'a, 'b>, data: CR)
where
    CR: CompositeRenderer + Send + Sync,
{
    builder.install_resource(data);
    builder.install_system(CompositeTransformSystem, "transform", &[]);
    builder.install_thread_local_system(CompositeRendererSystem::<CR>::default());
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(png_image_asset_protocol::PngImageAssetProtocol);
}
