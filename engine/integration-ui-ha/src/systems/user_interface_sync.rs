use crate::components::*;
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;

pub type HaUserInterfaceSyncSystemResources<'a> = (
    WorldRef,
    &'a ImageResourceMapping,
    &'a MaterialResourceMapping,
    Comp<&'a mut HaUserInterfaceSync>,
);

pub fn ha_user_interface_sync_system(universe: &mut Universe) {
    let (world, image_mapping, material_mapping, ..) =
        universe.query_resources::<HaUserInterfaceSyncSystemResources>();

    for (_, sync) in world.query::<&mut HaUserInterfaceSync>().iter() {
        sync.update_references(&material_mapping, &image_mapping);
    }
}
