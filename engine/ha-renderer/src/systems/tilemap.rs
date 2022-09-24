use crate::{
    components::{
        material_instance::HaMaterialInstance, mesh_instance::HaMeshInstance,
        tilemap_instance::HaTileMapInstance,
    },
    ha_renderer::HaRenderer,
    image::{ImageFiltering, ImageId, ImageReference, VirtualImageSource},
    material::{
        common::MaterialValue,
        domains::surface::{tilemap::SurfaceTileMapFactory, SurfaceVertexPT},
    },
    mesh::{Mesh, MeshId, MeshReference},
};
use core::ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaTileMapSystemCache {
    meshes: HashMap<Entity, MeshId>,
}

pub type HaTileMapSystemResources<'a> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a EntityChanges,
    &'a mut HaTileMapSystemCache,
    Comp<&'a mut HaTileMapInstance>,
    Comp<&'a mut HaMeshInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_tilemap_system(universe: &mut Universe) {
    let (world, mut renderer, changes, mut cache, ..) =
        universe.query_resources::<HaTileMapSystemResources>();

    for entity in changes.despawned() {
        if let Some(id) = cache.meshes.remove(&entity) {
            let _ = renderer.remove_mesh(id);
        }
    }

    for (entity, (tilemap, mesh, material)) in world
        .query::<(
            &mut HaTileMapInstance,
            &mut HaMeshInstance,
            &mut HaMaterialInstance,
        )>()
        .iter()
    {
        if tilemap.dirty || !cache.meshes.contains_key(&entity) {
            mesh.reference = MeshReference::None;
            if let Ok(factory) =
                SurfaceTileMapFactory::factory::<SurfaceVertexPT>(tilemap, &renderer.virtual_images)
            {
                let image_id = match renderer
                    .virtual_images
                    .get_named(tilemap.atlas())
                    .unwrap()
                    .source()
                {
                    VirtualImageSource::Image(image_id) => Some(*image_id),
                    _ => None,
                };
                if let Some(image_id) = image_id {
                    if let Some(id) = cache.meshes.get(&entity) {
                        if let Some(m) = renderer.mesh_mut(*id) {
                            m.set_vertex_storage_all(tilemap.change_frequency().into());
                            m.set_index_storage(tilemap.change_frequency().into());
                            if factory.write_into(m).is_ok() {
                                tilemap.dirty = false;
                                mesh.reference = MeshReference::Id(*id);
                                set_material_sampler(material, image_id, tilemap.filtering);
                            }
                        }
                    } else {
                        let mut m = Mesh::new(factory.layout().to_owned());
                        m.set_vertex_storage_all(tilemap.change_frequency().into());
                        m.set_index_storage(tilemap.change_frequency().into());
                        if factory.write_into(&mut m).is_ok() {
                            if let Ok(id) = renderer.add_mesh(m) {
                                tilemap.dirty = false;
                                mesh.reference = MeshReference::Id(id);
                                set_material_sampler(material, image_id, tilemap.filtering);
                                cache.meshes.insert(entity, id);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn set_material_sampler(material: &mut HaMaterialInstance, id: ImageId, filtering: ImageFiltering) {
    material.values.insert(
        "mainImage".to_owned(),
        MaterialValue::Sampler2d {
            reference: ImageReference::Id(id),
            filtering,
        },
    );
}
