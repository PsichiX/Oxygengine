pub mod asset;
pub mod asset_pack_preloader;
pub mod database;
pub mod protocol;
pub mod protocols;
pub mod system;

pub mod prelude {
    pub use super::{
        asset::*, asset_pack_preloader::*, database::*, protocol::*, protocols::prelude::*,
        protocols::*, system::*,
    };
}

use crate::{
    app::AppBuilder,
    assets::{
        database::AssetsDatabase,
        protocols::{
            binary::BinaryAssetProtocol, localization::LocalizationAssetProtocol,
            pack::PackAssetProtocol, prefab::PrefabAssetProtocol, set::SetAssetProtocol,
            text::TextAssetProtocol, yaml::YamlAssetProtocol,
        },
        system::AssetsSystem,
    },
    fetch::FetchEngine,
    localization::LocalizationSystem,
};

pub fn bundle_installer<'a, 'b, FE: 'static, ADS>(
    builder: &mut AppBuilder<'a, 'b>,
    (fetch_engine, mut assets_database_setup): (FE, ADS),
) where
    FE: FetchEngine,
    ADS: FnMut(&mut AssetsDatabase),
{
    let mut database = AssetsDatabase::new(fetch_engine);
    database.register(PackAssetProtocol);
    database.register(BinaryAssetProtocol);
    database.register(TextAssetProtocol);
    database.register(YamlAssetProtocol);
    database.register(SetAssetProtocol);
    database.register(PrefabAssetProtocol);
    database.register(LocalizationAssetProtocol);
    assets_database_setup(&mut database);
    builder.install_resource(database);
    builder.install_thread_local_system(AssetsSystem);
    builder.install_thread_local_system(LocalizationSystem::default());
}
