use crate::components::{
    player::{Player, PlayerType},
    speed::Speed,
};
use oxygengine::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use std::f64::consts::PI;

pub struct GameState;

impl GameState {
    fn create_map(world: &mut World, token: StateToken) {
        world
            .create_entity()
            .with(CompositeRenderable(().into()))
            .with(CompositeRenderDepth(-101.0))
            .with(CompositeTransform::default())
            .with(CompositeMapChunk::new("map.map".into(), "ground".into()))
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(().into()))
            .with(CompositeRenderDepth(-100.0))
            .with(CompositeTransform::default())
            .with(CompositeMapChunk::new("map.map".into(), "roads".into()))
            .with(NonPersistent(token))
            .build();
    }

    fn create_trees(world: &mut World, token: StateToken) {
        let mut rng = thread_rng();

        let trees_meta = {
            let assets = world.read_resource::<AssetsDatabase>();
            let asset = assets.asset_by_path("map://map.map").unwrap();
            let map = asset.get::<MapAsset>().unwrap().map();
            let y_treshold = (map.rows * map.tile_height) as isize / 2;
            let spawns = map.layer_by_name("spawns").unwrap().data.objects().unwrap();
            spawns
                .into_iter()
                .flat_map(|spawn| {
                    if !spawn.visible {
                        return vec![];
                    }
                    match spawn.object_type.as_str() {
                        "random" => {
                            let x_range = Uniform::from(spawn.x..(spawn.x + spawn.width as isize));
                            let y_range = Uniform::from(spawn.y..(spawn.y + spawn.height as isize));
                            let r_range = Uniform::from(0.0..PI);
                            let t_range_grass = Uniform::from(0..4);
                            let t_range_sand = Uniform::from(4..6);
                            let place_area = spawn.width * spawn.height;
                            let object_area = 128 * 128;
                            let count = place_area / object_area;
                            (0..count)
                                .map(|_| {
                                    let x = x_range.sample(&mut rng);
                                    let y = y_range.sample(&mut rng);
                                    let r = r_range.sample(&mut rng);
                                    let t = if y > y_treshold {
                                        t_range_sand.sample(&mut rng)
                                    } else {
                                        t_range_grass.sample(&mut rng)
                                    };
                                    (x, y, r, t)
                                })
                                .collect::<Vec<_>>()
                        }
                        _ => vec![],
                    }
                })
                .collect::<Vec<_>>()
        };

        for (x, y, r, t) in trees_meta {
            let (tree_type, depth) = match t {
                0 => ("treeGreen_large.png", 100.0),
                1 => ("treeGreen_small.png", 50.0),
                2 => ("treeBrown_large.png", 100.0),
                3 => ("treeBrown_small.png", 50.0),
                4 => ("barricadeMetal.png", 10.0),
                5 => ("barricadeWood.png", 10.0),
                _ => unreachable!(),
            };
            world
                .create_entity()
                .with(CompositeRenderable(().into()))
                .with(CompositeRenderDepth(depth))
                .with(
                    CompositeSprite::new("sprites.0.json".into(), tree_type.into())
                        .align(0.5.into()),
                )
                .with(CompositeTransform::default())
                .with(RigidBody2d::new(
                    RigidBodyDesc::new()
                        .translation(Vector::new(x as f64, y as f64))
                        .rotation(r),
                ))
                .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                    Ball::new(32.0),
                ))))
                .with(Collider2dBody::Me)
                .with(NonPersistent(token))
                .build();
        }
    }

    fn create_players(world: &mut World, token: StateToken) {
        let tanks_meta = {
            let assets = world.read_resource::<AssetsDatabase>();
            let asset = assets.asset_by_path("map://map.map").unwrap();
            let map = asset.get::<MapAsset>().unwrap().map();
            let spawns = map.layer_by_name("spawns").unwrap().data.objects().unwrap();
            spawns
                .into_iter()
                .filter_map(|spawn| {
                    if !spawn.visible {
                        return None;
                    }
                    match spawn.object_type.as_str() {
                        "player" => {
                            let x = spawn.x + spawn.width as isize / 2;
                            let y = spawn.y + spawn.height as isize / 2;
                            let (t, r) = if spawn.name == "north" {
                                (PlayerType::North, PI * 0.0)
                            } else {
                                (PlayerType::South, PI * 1.0)
                            };
                            Some((x, y, r, t))
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        };

        for (x, y, r, t) in tanks_meta {
            let tank_type = match t {
                PlayerType::North => "tank_red.png",
                PlayerType::South => "tank_blue.png",
            };
            world
                .create_entity()
                .with(CompositeRenderable(().into()))
                .with(CompositeRenderDepth(25.0))
                .with(
                    CompositeSprite::new("sprites.0.json".into(), tank_type.into())
                        .align(0.5.into()),
                )
                .with(CompositeTransform::default())
                .with(RigidBody2d::new(
                    RigidBodyDesc::new()
                        .translation(Vector::new(x as f64, y as f64))
                        .rotation(r),
                ))
                .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                    Ball::new(32.0),
                ))))
                .with(Collider2dBody::Me)
                .with(NonPersistent(token))
                .with(Speed(10.0))
                .with(Player(t))
                .build();
        }
    }
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        let token = world.read_resource::<AppLifeCycle>().current_state_token();

        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(2432.0.into()).with_translation(1216.0.into()))
            .with(NonPersistent(token))
            .build();

        Self::create_map(world, token);
        Self::create_trees(world, token);
        Self::create_players(world, token);
    }

    // fn on_process(&mut self, world: &mut World) -> StateChange {
    //     let dt = world.read_resource::<AppLifeCycle>().delta_time_seconds();
    //     info!("=== DT: {} | FRAME TIME: {}s", dt, 1.0 / dt);
    //     StateChange::None
    // }
}
