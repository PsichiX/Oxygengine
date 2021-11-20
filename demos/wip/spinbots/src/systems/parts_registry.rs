use crate::{asset_protocols::part::*, resources::parts_registry::*};
use oxygengine::prelude::*;

pub type PartsRegistrySystemResources<'a> = (&'a AssetsDatabase, &'a mut PartsRegistry);

pub fn parts_registry_system(universe: &mut Universe) {
    let (assets, mut parts) = universe.query_resources::<PartsRegistrySystemResources>();

    for id in assets.lately_loaded_protocol("part") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded part asset");
        let name = asset
            .path()
            .split('/')
            .nth(1)
            .expect("part name not found")
            .to_owned();
        let part = asset
            .get::<PartAsset>()
            .cloned()
            .expect("trying to use non-part asset");
        parts.register(id, name, part);
    }
    for id in assets.lately_unloaded_protocol("part") {
        parts.unregister(*id);
    }
}
