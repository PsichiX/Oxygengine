use crate::{
    components::{
        immediate_batch::HaImmediateBatch, material_instance::HaMaterialInstance,
        mesh_instance::HaMeshInstance,
    },
    ha_renderer::HaRenderer,
    material::domains::surface::SurfaceDomain,
    mesh::{BufferStorage, Mesh, MeshId, MeshInstanceReference},
};
use core::ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaImmediateBatchSystemCache {
    meshes: HashMap<Entity, MeshId>,
}

pub type HaImmediateBatchSystemResources<'a, V> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a EntityChanges,
    &'a mut HaImmediateBatchSystemCache,
    Comp<&'a mut HaImmediateBatch<V>>,
    Comp<&'a mut HaMeshInstance>,
    Comp<&'a HaMaterialInstance>,
);

pub fn ha_immediate_batch_system<V>(universe: &mut Universe)
where
    V: SurfaceDomain + Default + Copy + Send + Sync + 'static,
{
    let (world, mut renderer, changes, mut cache, ..) =
        universe.query_resources::<HaImmediateBatchSystemResources<V>>();

    for entity in changes.despawned() {
        if let Some(id) = cache.meshes.remove(&entity) {
            let _ = renderer.remove_mesh(id);
        }
    }

    for (entity, (batch, mesh)) in world
        .query::<(&mut HaImmediateBatch<V>, &mut HaMeshInstance)>()
        .with::<HaMaterialInstance>()
        .iter()
    {
        mesh.reference = MeshInstanceReference::None;
        if let Ok(factory) = batch.factory.factory() {
            if let Some(id) = cache.meshes.get(&entity) {
                if let Some(m) = renderer.mesh_mut(*id) {
                    if factory.write_into(m).is_ok() {
                        mesh.reference = MeshInstanceReference::Id(*id);
                    }
                }
            } else {
                let mut m = Mesh::new(factory.layout().to_owned());
                m.set_regenerate_bounds(false);
                m.set_vertex_storage_all(BufferStorage::Dynamic);
                m.set_index_storage(BufferStorage::Dynamic);
                if factory.write_into(&mut m).is_ok() {
                    if let Ok(id) = renderer.add_mesh(m) {
                        mesh.reference = MeshInstanceReference::Id(id);
                        cache.meshes.insert(entity, id);
                    }
                }
            }
        }
        batch.factory.clear();
    }
}
