use crate::{
    asset_protocols::font::FontAsset,
    components::{
        material_instance::HaMaterialInstance, mesh_instance::HaMeshInstance,
        text_instance::HaTextInstance,
    },
    constants::material_uniforms::*,
    ha_renderer::HaRenderer,
    image::{ImageReference, ImageResourceMapping},
    material::{
        common::MaterialValue,
        domains::surface::{text::SurfaceTextFactory, SurfaceVertexText},
    },
    mesh::{Mesh, MeshId, MeshReference},
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaFontSystemCache {
    fonts_map: HashMap<String, AssetId>,
    fonts_table: HashMap<AssetId, String>,
    meshes: HashMap<Entity, MeshId>,
}

pub type HaFontSystemResources<'a> = (
    WorldRef,
    &'a mut HaRenderer,
    &'a AssetsDatabase,
    &'a EntityChanges,
    &'a ImageResourceMapping,
    &'a mut HaFontSystemCache,
    Comp<&'a mut HaTextInstance>,
    Comp<&'a mut HaMeshInstance>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn ha_font_system(universe: &mut Universe) {
    let (world, mut renderer, assets, changes, image_mapping, mut cache, ..) =
        universe.query_resources::<HaFontSystemResources>();

    for id in assets.lately_loaded_protocol("font") {
        if let Some(asset) = assets.asset_by_id(*id) {
            if asset.is::<FontAsset>() {
                cache.fonts_map.insert(asset.path().to_owned(), *id);
                cache.fonts_table.insert(*id, asset.path().to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("font") {
        if let Some(name) = cache.fonts_table.remove(id) {
            cache.fonts_map.remove(&name);
        }
    }

    for entity in changes.despawned() {
        if let Some(id) = cache.meshes.remove(&entity) {
            let _ = renderer.remove_mesh(id);
        }
    }

    for (entity, (text, mesh, material)) in world
        .query::<(
            &mut HaTextInstance,
            &mut HaMeshInstance,
            &mut HaMaterialInstance,
        )>()
        .iter()
    {
        if text.dirty || !cache.meshes.contains_key(&entity) {
            mesh.reference = MeshReference::None;
            if let Some(id) = cache.fonts_map.get(text.font()) {
                if let Some(asset) = assets.asset_by_id(*id) {
                    if let Some(asset) = asset.get::<FontAsset>() {
                        if let Ok(factory) =
                            SurfaceTextFactory::factory::<SurfaceVertexText>(text, asset)
                        {
                            if let Some(id) = cache.meshes.get(&entity) {
                                if let Some(m) = renderer.mesh_mut(*id) {
                                    m.set_vertex_storage_all(text.change_frequency().into());
                                    m.set_index_storage(text.change_frequency().into());
                                    if factory.write_into(m).is_ok() {
                                        text.dirty = false;
                                        mesh.reference = MeshReference::Id(*id);
                                        set_material_sampler(material, asset, &image_mapping);
                                    }
                                }
                            } else {
                                let mut m = Mesh::new(factory.layout().to_owned());
                                m.set_vertex_storage_all(text.change_frequency().into());
                                m.set_index_storage(text.change_frequency().into());
                                if factory.write_into(&mut m).is_ok() {
                                    if let Ok(id) = renderer.add_mesh(m) {
                                        text.dirty = false;
                                        mesh.reference = MeshReference::Id(id);
                                        set_material_sampler(material, asset, &image_mapping);
                                        cache.meshes.insert(entity, id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn set_material_sampler(
    material: &mut HaMaterialInstance,
    font: &FontAsset,
    image_mapping: &ImageResourceMapping,
) {
    if let Some((_, id)) = font.pages_image_assets.get(0) {
        if let Some(id) = image_mapping.resource_by_asset(*id) {
            material.values.insert(
                MAIN_IMAGE_NAME.to_owned(),
                MaterialValue::Sampler2dArray {
                    reference: ImageReference::Id(id),
                    filtering: font.filtering,
                },
            );
        }
    }
}
