pub mod asset;
pub mod database;
pub mod protocol;
pub mod system;

use crate::{app::AppBuilder, assets::system::AssetsSystem, fetch::FetchEngine};

pub fn bundle_installer<'a, 'b, FE: 'static>(builder: &mut AppBuilder<'a, 'b>, data: FE)
where
    FE: FetchEngine,
{
    builder.install_resource(data);
    builder.install_thread_local_system(AssetsSystem);
}
