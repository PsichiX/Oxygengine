#![allow(clippy::many_single_char_names)]

use crate::{
    component::{
        CompositeAnimation, CompositeCamera, CompositeEffect, CompositeRenderAlpha,
        CompositeRenderDepth, CompositeRenderable, CompositeRenderableStroke, CompositeSprite,
        CompositeTilemap, CompositeTransform, CompositeVisibility, TileCell,
    },
    composite_renderer::{Command, CompositeRenderer, Image, Rectangle, Renderable, Stats},
    math::{Grid2d, Mat2d, Rect, Scalar},
    resource::CompositeTransformRes,
    sprite_sheet_asset_protocol::SpriteSheetAsset,
    tileset_asset_protocol::{TilesetAsset, TilesetInfo},
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetID, database::AssetsDatabase},
    ecs::{Entities, Entity, Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage},
    hierarchy::{HierarchyRes, Parent, Tag},
};
use std::{collections::HashMap, marker::PhantomData};

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
        ReadStorage<'s, CompositeRenderDepth>,
        ReadStorage<'s, CompositeRenderAlpha>,
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
            depths,
            alphas,
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

        if let Some(color) = renderer.state().clear_color {
            let result = renderer.execute(vec![Command::Draw(Renderable::Rectangle(Rectangle {
                color,
                rect: [0.0, 0.0, w, h].into(),
            }))]);
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
                if visible {
                    let depth = if let Some(depth) = depths.get(entity) {
                        depth.0
                    } else {
                        0.0
                    };
                    Some((depth, camera, transform))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        sorted_cameras.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (_, camera, camera_transform) in sorted_cameras {
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
                    if visible {
                        let depth = if let Some(depth) = depths.get(entity) {
                            depth.0
                        } else {
                            0.0
                        };
                        Some((depth, renderable, transform, entity))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let camera_matrix = camera.view_matrix(&camera_transform, [w, h].into());
            let commands = std::iter::once(Command::Store)
                .chain(std::iter::once({
                    let [a, b, c, d, e, f] = camera_matrix.0;
                    Command::Transform(a, b, c, d, e, f)
                }))
                .chain(
                    sorted
                        .iter()
                        .flat_map(|(_, renderable, transform, entity)| {
                            let [a, b, c, d, e, f] = transform.0;
                            vec![
                                Command::Store,
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
                    if let Renderable::Image(image) = &mut renderable.0 {
                        if let Some(name) = self.images_cache.get(sheet) {
                            if let Some(frames) = self.frames_cache.get(sheet) {
                                image.image = name.clone().into();
                                if let Some(frame) = frames.get(frame) {
                                    image.source = Some(*frame);
                                }
                                sprite.dirty = false;
                            }
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
                    if let Renderable::Commands(commands) = &mut renderable.0 {
                        if let Some(name) = self.images_cache.get(tileset) {
                            *commands = if let Some(info) = self.infos_cache.get(tileset) {
                                Self::build_commands(name, info, tilemap.grid())
                            } else {
                                vec![]
                            };
                            tilemap.dirty = false;
                        }
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
        let mut result = Vec::with_capacity(2 + grid.len());
        result.push(Command::Store);
        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                if let Some(cell) = grid.get(col, row) {
                    if !cell.visible {
                        continue;
                    }
                    if let Some(frame) = info.frame(cell.col, cell.row) {
                        if cell.is_abnormal() {
                            let (a, b, c, d, e, f) = cell.matrix().into();
                            result.push(Command::Store);
                            result.push(Command::Transform(a, b, c, d, e, f));
                        }
                        result.push(Command::Draw(
                            Image {
                                image: name.to_owned().into(),
                                source: Some(frame),
                                destination: Some(cell.destination(col, row, frame.w, frame.h)),
                                alignment: 0.0.into(),
                            }
                            .into(),
                        ));
                        if cell.is_abnormal() {
                            result.push(Command::Restore);
                        }
                    }
                }
            }
        }
        result.push(Command::Restore);
        result
    }
}

pub struct CompositeAnimationSystem;

impl<'s> System<'s> for CompositeAnimationSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        WriteStorage<'s, CompositeSprite>,
        WriteStorage<'s, CompositeAnimation>,
    );

    fn run(&mut self, (lifecycle, mut sprites, mut animations): Self::SystemData) {
        let dt = lifecycle.delta_time_seconds() as Scalar;
        for (sprite, animation) in (&mut sprites, &mut animations).join() {
            if animation.dirty {
                animation.dirty = false;
                if let Some((name, phase, _, _)) = &animation.current {
                    if let Some(anim) = animation.animations.get(name) {
                        if let Some(frame) = anim.frames.get(*phase as usize) {
                            sprite.set_sheet_frame(Some((anim.sheet.clone(), frame.clone())));
                        }
                    }
                }
            }
            animation.process(dt);
        }
    }
}
