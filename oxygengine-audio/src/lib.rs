extern crate oxygengine_core as core;

pub mod audio_asset_protocol;
pub mod component;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{audio_asset_protocol::*, component::*, resource::*, system::*};
}

use crate::{resource::Audio, system::AudioSystem};
use core::{app::AppBuilder, assets::database::AssetsDatabase};

pub fn bundle_installer<'a, 'b, A>(builder: &mut AppBuilder<'a, 'b>, data: A)
where
    A: Audio + 'static,
{
    builder.install_resource(data);
    builder.install_system(AudioSystem::<A>::default(), "audio", &[]);
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(audio_asset_protocol::AudioAssetProtocol);
}
