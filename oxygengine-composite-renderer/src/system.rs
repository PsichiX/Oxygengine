#![allow(clippy::many_single_char_names)]

use crate::{
    component::{
        CompositeCamera, CompositeCameraAlignment, CompositeEffect, CompositeMapChunk,
        CompositeMesh, CompositeRenderAlpha, CompositeRenderDepth, CompositeRenderLayer,
        CompositeRenderable, CompositeRenderableStroke, CompositeSprite, CompositeSpriteAnimation,
        CompositeSurfaceCache, CompositeTilemap, CompositeTilemapAnimation, CompositeTransform,
        CompositeUiElement, CompositeVisibility, TileCell,
    },
    composite_renderer::{Command, CompositeRenderer, Image, Renderable, Stats, Triangles},
    map_asset_protocol::{Map, MapAsset},
    math::{Mat2d, Rect, Vec2},
    mesh_asset_protocol::{Mesh, MeshAsset, MeshVertex, SubMesh},
    resource::{
        CompositeCameraCache, CompositeTransformRes, CompositeUiInteractibles, CompositeUiThemes,
    },
    sprite_sheet_asset_protocol::SpriteSheetAsset,
    tileset_asset_protocol::{TilesetAsset, TilesetInfo},
    ui_theme_asset_protocol::UiThemeAsset,
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetID, database::AssetsDatabase},
    ecs::{
        storage::ComponentEvent, Entities, Entity, Join, Read, ReadExpect, ReadStorage, ReaderId,
        System, World, Write, WriteStorage,
    },
    hierarchy::{HierarchyRes, Name, Parent, Tag},
    Scalar,
};
use std::{cmp::Ordering, collections::HashMap, marker::PhantomData};
use utils::grid_2d::Grid2d;

pub struct CompositeTransformSystem;

impl<'s> System<'s> for CompositeTransformSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, CompositeTransform>,
        ReadExpect<'s, HierarchyRes>,
        Write<'s, CompositeTransformRes>,
    );

    fn run(
        &mut self,
        (entities, parents, transforms, hierarchy, mut transform_res): Self::SystemData,
    ) {
        let hierarchy = &hierarchy;
        let mut transform_res = &mut transform_res;
        transform_res.clear();
        for (entity, transform, _) in (&entities, &transforms, !&parents).join() {
            transform_res.add(entity, transform.matrix());
            for child in hierarchy.children(entity) {
                add_matrix(
                    *child,
                    &transforms,
                    transform.matrix(),
                    hierarchy,
                    &mut transform_res,
                );
            }
        }
    }
}

fn add_matrix<'s>(
    child: Entity,
    transforms: &ReadStorage<'s, CompositeTransform>,
    root_matrix: Mat2d,
    hierarchy: &HierarchyRes,
    result: &mut CompositeTransformRes,
) {
    if let Some(transform) = transforms.get(child) {
        let mat = root_matrix * transform.matrix();
        result.add(child, mat);
        for child in hierarchy.children(child) {
            add_matrix(*child, transforms, mat, hierarchy, result);
        }
    }
}

pub struct CompositeCameraCacheSystem<CR>
where
    CR: CompositeRenderer,
{
    _phantom: PhantomData<CR>,
}

impl<CR> Default for CompositeCameraCacheSystem<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<'s, CR> System<'s> for CompositeCameraCacheSystem<CR>
where
    CR: CompositeRenderer + Send + Sync + 'static,
{
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Read<'s, CR>>,
        Write<'s, CompositeCameraCache>,
        Entities<'s>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeTransform>,
    );

    fn run(&mut self, (renderer, mut cache, entities, cameras, transforms): Self::SystemData) {
        if let Some(renderer) = renderer {
            let view_size = renderer.view_size();
            cache.last_view_size = view_size;
            cache.world_transforms = (&entities, &cameras, &transforms)
                .join()
                .map(|(entity, camera, transform)| {
                    (entity, camera.view_matrix(transform, view_size))
                })
                .collect::<HashMap<_, _>>();
            cache.world_inverse_transforms = cache
                .world_transforms
                .iter()
                .filter_map(|(k, v)| v.inverse().map(|v| (*k, v)))
                .collect::<HashMap<_, _>>();
        }
    }
}

pub struct CompositeRendererSystem<CR>
where
    CR: CompositeRenderer,
{
    _phantom: PhantomData<CR>,
}

impl<CR> Default for CompositeRendererSystem<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'s, CR> System<'s> for CompositeRendererSystem<CR>
where
    CR: CompositeRenderer + Send + Sync + 'static,
{
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Write<'s, CR>>,
        Entities<'s>,
        ReadExpect<'s, AppLifeCycle>,
        Option<Read<'s, AssetsDatabase>>,
        Read<'s, CompositeTransformRes>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeVisibility>,
        ReadStorage<'s, CompositeRenderable>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, CompositeRenderLayer>,
        ReadStorage<'s, CompositeRenderDepth>,
        ReadStorage<'s, CompositeRenderAlpha>,
        ReadStorage<'s, CompositeCameraAlignment>,
        ReadStorage<'s, CompositeRenderableStroke>,
        ReadStorage<'s, CompositeEffect>,
        ReadStorage<'s, Tag>,
    );

    fn run(
        &mut self,
        (
            renderer,
            entities,
            lifecycle,
            assets,
            transform_res,
            cameras,
            visibilities,
            renderables,
            transforms,
            layers,
            depths,
            alphas,
            alignments,
            strokes,
            effects,
            tags,
        ): Self::SystemData,
    ) {
        if renderer.is_none() {
            return;
        }

        let renderer: &mut CR = &mut renderer.unwrap();
        if let Some(assets) = &assets {
            renderer.update_cache(assets);
        }
        renderer.update_state();
        let (w, h) = {
            let r = renderer.view_size();
            (r.x, r.y)
        };
        let mut stats = Stats::default();
        stats.view_size = renderer.view_size();
        stats.images_count = renderer.images_count();
        stats.fontfaces_count = renderer.fontfaces_count();
        stats.surfaces_count = renderer.surfaces_count();

        if let Some(color) = renderer.state().clear_color {
            let result =
                renderer.execute(vec![Command::Draw(Renderable::FullscreenRectangle(color))]);
            if let Ok((render_ops, renderables)) = result {
                stats.render_ops += render_ops;
                stats.renderables += renderables;
            }
        }

        let mut sorted_cameras = (&entities, &cameras, &transforms)
            .join()
            .filter_map(|(entity, camera, transform)| {
                let visible = if let Some(visibility) = visibilities.get(entity) {
                    visibility.0
                } else {
                    true
                };
                let alpha = alphas.get(entity).map(|alpha| alpha.0);
                let alpha_visible = if let Some(alpha) = alpha {
                    alpha > 0.0
                } else {
                    true
                };
                if visible && alpha_visible {
                    let layer = if let Some(layer) = layers.get(entity) {
                        layer.0
                    } else {
                        0
                    };
                    let depth = if let Some(depth) = depths.get(entity) {
                        depth.0
                    } else {
                        0.0
                    };
                    Some((layer, depth, alpha, camera, transform))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        sorted_cameras.sort_by(|a, b| {
            let layer = a.0.partial_cmp(&b.0).unwrap();
            if layer == Ordering::Equal {
                a.1.partial_cmp(&b.1).unwrap()
            } else {
                layer
            }
        });

        for (_, _, camera_alpha, camera, camera_transform) in sorted_cameras {
            let mut sorted = (&entities, &renderables, transform_res.read())
                .join()
                .filter(|(entity, _, _)| {
                    camera.tags.is_empty()
                        || tags
                            .get(*entity)
                            .map_or(false, |tag| camera.tags.contains(&tag.0))
                })
                .filter_map(|(entity, renderable, transform)| {
                    let visible = if let Some(visibility) = visibilities.get(entity) {
                        visibility.0
                    } else {
                        true
                    };
                    let alpha_visible = if let Some(alpha) = alphas.get(entity) {
                        alpha.0 > 0.0
                    } else {
                        true
                    };
                    if visible && alpha_visible {
                        let layer = if let Some(layer) = layers.get(entity) {
                            layer.0
                        } else {
                            0
                        };
                        let depth = if let Some(depth) = depths.get(entity) {
                            depth.0
                        } else {
                            0.0
                        };
                        Some((layer, depth, renderable, transform, entity))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            sorted.sort_by(|a, b| {
                let layer = a.0.partial_cmp(&b.0).unwrap();
                if layer == Ordering::Equal {
                    a.1.partial_cmp(&b.1).unwrap()
                } else {
                    layer
                }
            });

            let camera_matrix = camera.view_matrix(&camera_transform, [w, h].into());
            let commands = std::iter::once(Command::Store)
                .chain(std::iter::once({
                    let [a, b, c, d, e, f] = camera_matrix.0;
                    Command::Transform(a, b, c, d, e, f)
                }))
                .chain(std::iter::once(if let Some(alpha) = camera_alpha {
                    Command::Alpha(alpha)
                } else {
                    Command::None
                }))
                .chain(
                    sorted
                        .iter()
                        .flat_map(|(_, _, renderable, transform, entity)| {
                            let [a, b, c, d, e, f] = transform.0;
                            vec![
                                Command::Store,
                                if let Some(alignment) = alignments.get(*entity) {
                                    let p = Vec2::new(alignment.0.x * w, alignment.0.y * h);
                                    let [a, b, c, d, e, f] =
                                        ((!camera_matrix).unwrap() * Mat2d::translation(p)).0;
                                    Command::Transform(a, b, c, d, e, f)
                                } else {
                                    Command::None
                                },
                                Command::Transform(a, b, c, d, e, f),
                                if let Some(effect) = effects.get(*entity) {
                                    Command::Effect(effect.0)
                                } else {
                                    Command::None
                                },
                                if let Some(alpha) = alphas.get(*entity) {
                                    Command::Alpha(alpha.0)
                                } else {
                                    Command::None
                                },
                                if let Some(stroke) = strokes.get(*entity) {
                                    Command::Stroke(stroke.0, renderable.0.clone())
                                } else {
                                    Command::Draw(renderable.0.clone())
                                },
                                Command::Restore,
                            ]
                        }),
                )
                .chain(std::iter::once(Command::Restore));

            if let Ok((render_ops, renderables)) = renderer.execute(commands) {
                stats.render_ops += render_ops;
                stats.renderables += renderables;
            }
        }
        stats.delta_time = lifecycle.delta_time_seconds();
        stats.fps = 1.0 / stats.delta_time;
        renderer.state_mut().set_stats(stats);
    }
}

#[derive(Debug, Default)]
pub struct CompositeSpriteSheetSystem {
    images_cache: HashMap<String, String>,
    atlas_table: HashMap<AssetID, String>,
    frames_cache: HashMap<String, HashMap<String, Rect>>,
}

impl<'s> System<'s> for CompositeSpriteSheetSystem {
    type SystemData = (
        ReadExpect<'s, AssetsDatabase>,
        WriteStorage<'s, CompositeRenderable>,
        WriteStorage<'s, CompositeSprite>,
    );

    fn run(&mut self, (assets, mut renderables, mut sprites): Self::SystemData) {
        for id in assets.lately_loaded_protocol("atlas") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded atlas asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<SpriteSheetAsset>()
                .expect("trying to use non-atlas asset");
            let image = asset.info().meta.image_name();
            let frames = asset
                .info()
                .frames
                .iter()
                .map(|(k, v)| (k.to_owned(), v.frame))
                .collect();
            self.images_cache.insert(path.clone(), image);
            self.atlas_table.insert(id, path.clone());
            self.frames_cache.insert(path, frames);
        }
        for id in assets.lately_unloaded_protocol("atlas") {
            if let Some(path) = self.atlas_table.remove(id) {
                self.images_cache.remove(&path);
                self.frames_cache.remove(&path);
            }
        }

        for (renderable, sprite) in (&mut renderables, &mut sprites).join() {
            if sprite.dirty {
                if let Some((sheet, frame)) = sprite.sheet_frame() {
                    if let Some(name) = self.images_cache.get(sheet) {
                        if let Some(frames) = self.frames_cache.get(sheet) {
                            renderable.0 = Image {
                                image: name.clone().into(),
                                source: frames.get(frame).copied(),
                                destination: None,
                                alignment: sprite.alignment,
                            }
                            .into();
                            sprite.dirty = false;
                        }
                    }
                } else {
                    sprite.dirty = false;
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct CompositeTilemapSystem {
    images_cache: HashMap<String, String>,
    atlas_table: HashMap<AssetID, String>,
    infos_cache: HashMap<String, TilesetInfo>,
}

impl<'s> System<'s> for CompositeTilemapSystem {
    type SystemData = (
        ReadExpect<'s, AssetsDatabase>,
        WriteStorage<'s, CompositeRenderable>,
        WriteStorage<'s, CompositeTilemap>,
    );

    fn run(&mut self, (assets, mut renderables, mut tilemaps): Self::SystemData) {
        for id in assets.lately_loaded_protocol("tiles") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded tileset asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<TilesetAsset>()
                .expect("trying to use non-tileset asset");
            let image = asset.info().image_name();
            let info = asset.info().clone();
            self.images_cache.insert(path.clone(), image);
            self.atlas_table.insert(id, path.clone());
            self.infos_cache.insert(path, info);
        }
        for id in assets.lately_unloaded_protocol("tiles") {
            if let Some(path) = self.atlas_table.remove(id) {
                self.images_cache.remove(&path);
                self.infos_cache.remove(&path);
            }
        }

        for (renderable, tilemap) in (&mut renderables, &mut tilemaps).join() {
            if tilemap.dirty {
                if let Some(tileset) = tilemap.tileset() {
                    if let Some(name) = self.images_cache.get(tileset) {
                        let commands = if let Some(info) = self.infos_cache.get(tileset) {
                            Self::build_commands(name, info, tilemap.grid())
                        } else {
                            vec![]
                        };
                        renderable.0 = Renderable::Commands(commands);
                        tilemap.dirty = false;
                    }
                } else {
                    tilemap.dirty = false;
                }
            }
        }
    }
}

impl CompositeTilemapSystem {
    fn build_commands<'a>(
        name: &str,
        info: &TilesetInfo,
        grid: &Grid2d<TileCell>,
    ) -> Vec<Command<'a>> {
        let mut result = Vec::with_capacity(grid.len() * 4);
        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                if let Some(cell) = grid.get(col, row) {
                    if !cell.visible {
                        continue;
                    }
                    if let Some(frame) = info.frame(cell.col, cell.row) {
                        let (a, b, c, d, e, f) = cell.matrix(col, row, frame.w, frame.h).into();
                        result.push(Command::Store);
                        result.push(Command::Transform(a, b, c, d, e, f));
                        result.push(Command::Draw(
                            Image {
                                image: name.to_owned().into(),
                                source: Some(frame),
                                destination: None,
                                alignment: 0.0.into(),
                            }
                            .into(),
                        ));
                        result.push(Command::Restore);
                    }
                }
            }
        }
        result
    }
}

pub struct CompositeSpriteAnimationSystem;

impl<'s> System<'s> for CompositeSpriteAnimationSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Entities<'s>,
        WriteStorage<'s, CompositeSprite>,
        WriteStorage<'s, CompositeSpriteAnimation>,
        WriteStorage<'s, CompositeSurfaceCache>,
    );

    fn run(
        &mut self,
        (lifecycle, entities, mut sprites, mut animations, mut caches): Self::SystemData,
    ) {
        let dt = lifecycle.delta_time_seconds() as Scalar;
        for (entity, sprite, animation) in (&entities, &mut sprites, &mut animations).join() {
            if animation.dirty {
                animation.dirty = false;
                if let Some((name, phase, _, _)) = &animation.current {
                    if let Some(anim) = animation.animations.get(name) {
                        if let Some(frame) = anim.frames.get(*phase as usize) {
                            sprite.set_sheet_frame(Some((anim.sheet.clone(), frame.clone())));
                            if let Some(cache) = caches.get_mut(entity) {
                                cache.rebuild();
                            }
                        }
                    }
                }
            }
            animation.process(dt);
        }
    }
}

pub struct CompositeTilemapAnimationSystem;

impl<'s> System<'s> for CompositeTilemapAnimationSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Entities<'s>,
        WriteStorage<'s, CompositeTilemap>,
        WriteStorage<'s, CompositeTilemapAnimation>,
        WriteStorage<'s, CompositeSurfaceCache>,
    );

    fn run(
        &mut self,
        (lifecycle, entities, mut tilemaps, mut animations, mut caches): Self::SystemData,
    ) {
        let dt = lifecycle.delta_time_seconds() as Scalar;
        for (entity, tilemap, animation) in (&entities, &mut tilemaps, &mut animations).join() {
            if animation.dirty {
                animation.dirty = false;
                if let Some((name, phase, _, _)) = &animation.current {
                    if let Some(anim) = animation.animations.get(name) {
                        if let Some(frame) = anim.frames.get(*phase as usize) {
                            tilemap.set_tileset(Some(anim.tileset.clone()));
                            tilemap.set_grid(frame.clone());
                            if let Some(cache) = caches.get_mut(entity) {
                                cache.rebuild();
                            }
                        }
                    }
                }
            }
            animation.process(dt);
        }
    }
}

pub struct CompositeSurfaceCacheSystem<CR>
where
    CR: CompositeRenderer,
{
    cached_surfaces: HashMap<Entity, String>,
    reader_id: Option<ReaderId<ComponentEvent>>,
    _phantom: PhantomData<CR>,
}

impl<CR> Default for CompositeSurfaceCacheSystem<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            cached_surfaces: Default::default(),
            reader_id: None,
            _phantom: PhantomData,
        }
    }
}

impl<'s, CR> System<'s> for CompositeSurfaceCacheSystem<CR>
where
    CR: CompositeRenderer + 'static,
{
    type SystemData = (
        Entities<'s>,
        Option<Write<'s, CR>>,
        WriteStorage<'s, CompositeSurfaceCache>,
        WriteStorage<'s, CompositeRenderable>,
    );

    fn setup(&mut self, world: &mut World) {
        use core::ecs::SystemData;
        Self::SystemData::setup(world);
        self.reader_id =
            Some(WriteStorage::<CompositeSurfaceCache>::fetch(&world).register_reader());
    }

    fn run(&mut self, (entities, renderer, mut caches, mut renderables): Self::SystemData) {
        if renderer.is_none() {
            return;
        }

        let renderer: &mut CR = &mut renderer.unwrap();
        let events = caches.channel().read(self.reader_id.as_mut().unwrap());
        for event in events {
            if let ComponentEvent::Removed(index) = event {
                let found = self.cached_surfaces.iter().find_map(|(entity, name)| {
                    if entity.id() == *index {
                        Some((*entity, name.clone()))
                    } else {
                        None
                    }
                });
                if let Some((entity, name)) = found {
                    self.cached_surfaces.remove(&entity);
                    renderer.destroy_surface(&name);
                }
            }
        }

        for (entity, cache, renderable) in (&entities, &mut caches, &mut renderables).join() {
            if cache.dirty {
                cache.dirty = false;
                if !renderer.has_surface(cache.name()) {
                    renderer.create_surface(cache.name(), cache.width(), cache.height());
                    self.cached_surfaces.insert(entity, cache.name().to_owned());
                } else if let Some((width, height)) = renderer.get_surface_size(cache.name()) {
                    if width != cache.width() || height != cache.height() {
                        renderer.destroy_surface(cache.name());
                        renderer.create_surface(cache.name(), cache.width(), cache.height());
                        self.cached_surfaces.insert(entity, cache.name().to_owned());
                    }
                }
                let commands = vec![
                    Command::Store,
                    Command::Draw(renderable.0.clone()),
                    Command::Restore,
                ];
                if renderer.update_surface(cache.name(), commands).is_ok() {
                    renderable.0 = Image {
                        image: cache.name().to_owned().into(),
                        source: None,
                        destination: None,
                        alignment: 0.0.into(),
                    }
                    .into();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct CompositeMapSystem {
    maps_cache: HashMap<String, Map>,
    maps_table: HashMap<AssetID, String>,
}

impl<'s> System<'s> for CompositeMapSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, AssetsDatabase>,
        WriteStorage<'s, CompositeMapChunk>,
        WriteStorage<'s, CompositeRenderable>,
        WriteStorage<'s, CompositeSurfaceCache>,
    );

    fn run(
        &mut self,
        (entities, assets, mut chunks, mut renderables, mut caches): Self::SystemData,
    ) {
        for id in assets.lately_loaded_protocol("map") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded map asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<MapAsset>()
                .expect("trying to use non-map asset");
            let map = asset.map().clone();
            self.maps_cache.insert(path.clone(), map);
            self.maps_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("map") {
            if let Some(path) = self.maps_table.remove(id) {
                self.maps_cache.remove(&path);
            }
        }

        for (entity, chunk, renderable) in (&entities, &mut chunks, &mut renderables).join() {
            if chunk.dirty {
                if let Some(map) = self.maps_cache.get(chunk.map_name()) {
                    let commands = if let Some(commands) = map
                        .build_render_commands_from_layer_by_name(
                            chunk.layer_name(),
                            chunk.offset(),
                            chunk.size(),
                            &assets,
                        ) {
                        commands
                    } else {
                        vec![]
                    };
                    renderable.0 = Renderable::Commands(commands);
                    if let Some(cache) = caches.get_mut(entity) {
                        cache.rebuild();
                    }
                    chunk.dirty = false;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct CompositeMeshSystem {
    meshes_cache: HashMap<String, Mesh>,
    meshes_table: HashMap<AssetID, String>,
}

impl<'s> System<'s> for CompositeMeshSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, AssetsDatabase>,
        WriteStorage<'s, CompositeMesh>,
        WriteStorage<'s, CompositeRenderable>,
        WriteStorage<'s, CompositeSurfaceCache>,
    );

    fn run(
        &mut self,
        (entities, assets, mut meshes, mut renderables, mut caches): Self::SystemData,
    ) {
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
            self.meshes_cache.insert(path.clone(), mesh);
            self.meshes_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("mesh") {
            if let Some(path) = self.meshes_table.remove(id) {
                self.meshes_cache.remove(&path);
            }
        }

        for (entity, mesh, renderable) in (&entities, &mut meshes, &mut renderables).join() {
            if mesh.dirty_mesh || mesh.dirty_visuals {
                if let Some(asset) = self.meshes_cache.get(mesh.mesh()) {
                    if mesh.dirty_mesh {
                        if let Some(root) = &asset.rig {
                            mesh.setup_bones_from_rig(root);
                        }
                    }
                    if mesh.dirty_visuals {
                        let vertices = if let Some(root) = &asset.rig {
                            mesh.rebuild_model_space(root);
                            Self::build_skined_vertices(&asset.vertices, mesh)
                        } else {
                            Self::build_vertices(&asset.vertices)
                        };
                        let commands = asset
                            .submeshes
                            .iter()
                            .zip(mesh.images().iter())
                            .map(|(submesh, image)| {
                                Self::build_command(&vertices, submesh, image.to_string())
                            })
                            .collect::<Vec<_>>();
                        renderable.0 = Renderable::Commands(commands);
                    }
                    if let Some(cache) = caches.get_mut(entity) {
                        cache.rebuild();
                    }
                    mesh.dirty_mesh = false;
                    mesh.dirty_visuals = false;
                }
            }
        }
    }
}

impl CompositeMeshSystem {
    fn build_skined_vertices(vertices: &[MeshVertex], mesh: &CompositeMesh) -> Vec<(Vec2, Vec2)> {
        vertices
            .iter()
            .map(|v| {
                let a = if let Some(m) = mesh.bones_model_space.get(&v.bone_info[0].name) {
                    *m * v.position
                } else {
                    v.position
                } * v.bone_info[0].weight;
                let b = if let Some(m) = mesh.bones_model_space.get(&v.bone_info[1].name) {
                    *m * v.position
                } else {
                    v.position
                } * v.bone_info[1].weight;
                let c = if let Some(m) = mesh.bones_model_space.get(&v.bone_info[2].name) {
                    *m * v.position
                } else {
                    v.position
                } * v.bone_info[2].weight;
                let d = if let Some(m) = mesh.bones_model_space.get(&v.bone_info[3].name) {
                    *m * v.position
                } else {
                    v.position
                } * v.bone_info[3].weight;
                (a + b + c + d, v.tex_coord)
            })
            .collect::<Vec<_>>()
    }

    fn build_vertices(vertices: &[MeshVertex]) -> Vec<(Vec2, Vec2)> {
        vertices
            .iter()
            .map(|v| (v.position, v.tex_coord))
            .collect::<Vec<_>>()
    }

    fn build_command<'a>(
        vertices: &[(Vec2, Vec2)],
        submesh: &SubMesh,
        image: String,
    ) -> Command<'a> {
        let triangles = Triangles {
            image: image.into(),
            color: Default::default(),
            vertices: vertices.to_vec(),
            faces: submesh.cached_faces().to_vec(),
        };
        Command::Draw(triangles.into())
    }
}

#[derive(Debug)]
pub struct CompositeUiSystem<CR>
where
    CR: CompositeRenderer,
{
    last_view_size: Vec2,
    _phantom: PhantomData<CR>,
}

impl<CR> Default for CompositeUiSystem<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            last_view_size: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'s, CR> System<'s> for CompositeUiSystem<CR>
where
    CR: CompositeRenderer + 'static,
{
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Read<'s, CR>>,
        ReadExpect<'s, AssetsDatabase>,
        Write<'s, CompositeUiInteractibles>,
        Write<'s, CompositeUiThemes>,
        WriteStorage<'s, CompositeUiElement>,
        WriteStorage<'s, CompositeRenderable>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, Name>,
    );

    fn run(
        &mut self,
        (
            renderer,
            assets,
            mut interactibles,
            mut themes,
            mut ui_elements,
            mut renderables,
            cameras,
            transforms,
            names,
        ): Self::SystemData,
    ) {
        for id in assets.lately_loaded_protocol("ui-theme") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded ui theme asset");
            let asset = asset
                .get::<UiThemeAsset>()
                .expect("trying to use non-ui-theme asset");
            for (key, value) in asset.get() {
                themes.themes.insert(key.clone().into(), value.clone());
            }
        }
        for id in assets.lately_unloaded_protocol("ui-theme") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded ui theme asset");
            let asset = asset
                .get::<UiThemeAsset>()
                .expect("trying to use non-ui-theme asset");
            for key in asset.get().keys() {
                themes.themes.remove(key.clone().as_str());
            }
        }

        if renderer.is_none() {
            return;
        }

        let renderer = renderer.unwrap();
        let view_size = renderer.view_size();
        // let force_update = (self.last_view_size - view_size).sqr_magnitude() > 1.0e-4;
        self.last_view_size = view_size;

        interactibles.bounding_boxes.clear();
        for (mut ui_element, mut renderable) in (&mut ui_elements, &mut renderables).join() {
            // TODO: add elements to interactibles while not rebuilding commands if not dirty.
            // if ui_element.dirty || force_update {
            if let Some(rect) = (&cameras, &names, &transforms)
                .join()
                .find_map(|(c, n, t)| {
                    if ui_element.camera_name == n.0 {
                        if let Some(inv_mat) = !c.view_matrix(t, view_size) {
                            let size = view_size * inv_mat;
                            Some(Rect {
                                x: 0.0,
                                y: 0.0,
                                w: size.x,
                                h: size.y,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            {
                let commands = ui_element
                    .build_commands(rect, &mut interactibles, &themes, &mut vec![])
                    .0;
                renderable.0 = Renderable::Commands(commands);
                ui_element.dirty = false;
            }
            // }
        }
    }
}
