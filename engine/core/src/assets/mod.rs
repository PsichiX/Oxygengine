pub mod asset;
pub mod asset_pack_preloader;
pub mod assets_preloader;
pub mod database;
pub mod protocol;
pub mod protocols;
pub mod system;

use crate::{
    app::AppBuilder,
    assets::{
        database::AssetsDatabase,
        protocols::{
            binary::BinaryAssetProtocol, json::JsonAssetProtocol,
            localization::LocalizationAssetProtocol, meta::MetaAssetProtocol,
            pack::PackAssetProtocol, prefab::PrefabAssetProtocol, text::TextAssetProtocol,
            toml::TomlAssetProtocol,
        },
        system::{assets_system, AssetsSystemResources},
    },
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    fetch::FetchEngine,
};

pub fn bundle_installer<PB, FE, ADS>(
    builder: &mut AppBuilder<PB>,
    (fetch_engine, mut assets_database_setup): (FE, ADS),
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    FE: FetchEngine + 'static,
    ADS: FnMut(&mut AssetsDatabase),
{
    let mut database = AssetsDatabase::new(fetch_engine);
    database.register(PackAssetProtocol);
    database.register(BinaryAssetProtocol);
    database.register(TextAssetProtocol);
    database.register(JsonAssetProtocol);
    database.register(TomlAssetProtocol);
    database.register(PrefabAssetProtocol);
    database.register(LocalizationAssetProtocol);
    database.register(MetaAssetProtocol);
    assets_database_setup(&mut database);
    builder.install_resource(database);
    builder.install_system::<AssetsSystemResources>("assets", assets_system, &[])?;
    Ok(())
}
