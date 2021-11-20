use crate::{
    component::{CompositeMesh, CompositeRenderable, CompositeSurfaceCache},
    composite_renderer::{Command, Mask, PathElement, Renderable, Triangles},
    math::Vec2,
    mesh_asset_protocol::{Mesh, MeshAsset, MeshVertex},
};
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeMeshSystemCache {
    meshes_cache: HashMap<String, Mesh>,
    meshes_table: HashMap<AssetId, String>,
}

pub type CompositeMeshSystemResources<'a> = (
    WorldRef,
    &'a AssetsDatabase,
    &'a mut CompositeMeshSystemCache,
    Comp<&'a mut CompositeMesh>,
    Comp<&'a mut CompositeRenderable>,
    Comp<&'a mut CompositeSurfaceCache>,
);

pub fn composite_mesh_system(universe: &mut Universe) {
    let (world, assets, mut cache, ..) = universe.query_resources::<CompositeMeshSystemResources>();

    for id in assets.lately_loaded_protocol("mesh") {
        let id = *id;
        let asset = assets
            .asset_by_id(id)
            .expect("trying to use not loaded mesh asset");
        let path = asset.path().to_owned();
        let asset = asset
            .get::<MeshAsset>()
            .expect("trying to use non-mesh asset");
        let mesh = asset.mesh().clone();
        cache.meshes_cache.insert(path.clone(), mesh);
        cache.meshes_table.insert(id, path);
    }
    for id in assets.lately_unloaded_protocol("mesh") {
        if let Some(path) = cache.meshes_table.remove(id) {
            cache.meshes_cache.remove(&path);
        }
    }

    for (_, (mesh, renderable, surface)) in world
        .query::<(
            &mut CompositeMesh,
            &mut CompositeRenderable,
            Option<&mut CompositeSurfaceCache>,
        )>()
        .iter()
    {
        if mesh.dirty_mesh || mesh.dirty_visuals {
            if let Some(r) = build_renderable(mesh, &cache.meshes_cache) {
                renderable.0 = r;
                if let Some(surface) = surface {
                    surface.rebuild();
                }
                mesh.dirty_mesh = false;
                mesh.dirty_visuals = false;
            }
        }
    }
}

fn build_renderable<'a>(
    mesh: &mut CompositeMesh,
    meshes: &HashMap<String, Mesh>,
) -> Option<Renderable<'a>> {
    if let Some(asset) = meshes.get(mesh.mesh()) {
        if mesh.dirty_mesh {
            if let Some(root) = &asset.rig {
                mesh.setup_bones_from_rig(root);
            }
        }
        if mesh.dirty_visuals {
            let vertices = if let Some(root) = &asset.rig {
                mesh.rebuild_model_space(root);
                build_skined_vertices(&asset.vertices, mesh)
            } else {
                build_vertices(&asset.vertices)
            };
            let masks = asset
                .masks
                .iter()
                .map(|indices| build_mask(&vertices, &indices.indices))
                .collect::<Vec<_>>();
            let mut meta = asset
                .submeshes
                .iter()
                .zip(mesh.materials().iter())
                .filter_map(|(submesh, material)| {
                    if material.alpha > 0.0 {
                        let triangles = Triangles {
                            image: material.image.to_string().into(),
                            color: Default::default(),
                            vertices: vertices.to_vec(),
                            faces: submesh.cached_faces().to_vec(),
                        };
                        let masks = submesh
                            .masks
                            .iter()
                            .map(|i| masks[*i].to_vec())
                            .collect::<Vec<_>>();
                        Some((triangles, material.alpha, material.order, masks))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            meta.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            let count = meta.len() * 4 + meta.iter().fold(0, |a, v| a + v.3.len());
            let mut commands = Vec::with_capacity(count);
            for (triangles, alpha, _, masks) in meta {
                commands.push(Command::Store);
                for mask in masks {
                    let mask = Mask { elements: mask };
                    commands.push(Command::Draw(mask.into()));
                }
                commands.push(Command::Alpha(alpha));
                commands.push(Command::Draw(triangles.into()));
                commands.push(Command::Restore);
            }
            return Some(Renderable::Commands(commands));
        }
    }
    None
}

fn build_skined_vertices(vertices: &[MeshVertex], mesh: &CompositeMesh) -> Vec<(Vec2, Vec2)> {
    vertices
        .iter()
        .map(|v| {
            let p = v.bone_info.iter().fold(Vec2::default(), |a, i| {
                let p = if let Some(m) = mesh.bones_model_space.get(&i.name) {
                    *m * v.position
                } else {
                    v.position
                };
                a + p * i.weight
            });
            (p, v.tex_coord)
        })
        .collect::<Vec<_>>()
}

fn build_vertices(vertices: &[MeshVertex]) -> Vec<(Vec2, Vec2)> {
    vertices
        .iter()
        .map(|v| (v.position, v.tex_coord))
        .collect::<Vec<_>>()
}

fn build_mask(vertices: &[(Vec2, Vec2)], indices: &[usize]) -> Vec<PathElement> {
    let mut result = Vec::with_capacity(indices.len());
    for index in indices {
        if *index == 0 {
            result.push(PathElement::MoveTo(vertices[*index].0));
        } else {
            result.push(PathElement::LineTo(vertices[*index].0));
        }
    }
    result
}
