extern crate oxygengine_core as core;

pub mod audio_asset_protocol;
pub mod component;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{audio_asset_protocol::*, component::*, resource::*, system::*};
}

use crate::{
    component::{AudioSource, AudioSourcePrefabProxy},
    resource::Audio,
    system::{audio_system, AudioSystemResources},
};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    prefab::PrefabManager,
};

pub fn bundle_installer<PB, A>(
    builder: &mut AppBuilder<PB>,
    data: A,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    A: Audio + 'static,
{
    builder.install_resource(data);
    builder.install_system::<AudioSystemResources<A>>("audio", audio_system::<A>, &[])?;
    Ok(())
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(audio_asset_protocol::AudioAssetProtocol);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory_proxy::<AudioSource, AudioSourcePrefabProxy>("AudioSource");
}
