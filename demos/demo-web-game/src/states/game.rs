use crate::{
    components::{
        ammo::Ammo,
        bonus::{Bonus, BonusType},
        bullet::Bullet,
        death::Death,
        follow::{Follow, FollowMode},
        health::Health,
        lifetime::Lifetime,
        player::{Player, PlayerType},
        speed::Speed,
    },
    resources::{
        globals::{GamePhase, Globals},
        spawner::{DespawnEffect, Spawn, Spawner},
        turn::{Timer, TurnManager},
    },
};
use oxygengine::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, RngCore,
};
use std::{collections::HashMap, f64::consts::PI};

const HALF_WALL_THICKNESS: f64 = 50.0;
const EXPLOSION_TIME: f64 = 0.5;

#[derive(Default)]
pub struct GameState {
    ui_timer: Option<Entity>,
    ui_player: Option<Entity>,
    ui_panel: Option<Entity>,
    ui_panel_text: Option<Entity>,
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        let token = world.read_resource::<AppLifeCycle>().current_state_token();

        let camera = world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect).tag("world".into()))
            .with(CompositeTransform::scale(720.0.into()).with_translation(1216.0.into()))
            .with(NonPersistent(token))
            .with(Follow(None, FollowMode::Delayed(0.98)))
            .with(AudioSource::from(
                AudioSourceConfig::new("music/strength-of-the-titans.ogg".into())
                    .streaming(true)
                    .volume(0.5)
                    .looped(true)
                    .play(true),
            ))
            .build();

        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect).tag("ui".into()))
            .with(CompositeTransform::scale(720.0.into()))
            .with(CompositeRenderDepth(1.0))
            .with(NonPersistent(token))
            .build();

        world.write_resource::<Globals>().start(camera);
        Self::create_map(world, token);
        Self::create_trees(world, token);
        Self::create_players(world, token);
        Self::create_bonuses(world, token);
        self.create_ui(world, token);
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        self.update_ui(world);
        Self::spawn_and_despawn(world);

        if world.read_resource::<Globals>().phase.is_restart() {
            StateChange::Swap(Box::new(GameState::default()))
        } else {
            StateChange::None
        }
    }

    fn on_exit(&mut self, world: &mut World) {
        world.write_resource::<Globals>().reset();
        world.write_resource::<TurnManager>().reset();
    }
}

impl GameState {
    fn create_map(world: &mut World, token: StateToken) {
        world
            .create_entity()
            .with(Tag("world".into()))
            .with(CompositeRenderable(().into()))
            .with(CompositeRenderDepth(-101.0))
            .with(CompositeTransform::default())
            .with(CompositeMapChunk::new("map.map".into(), "ground".into()))
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(Tag("world".into()))
            .with(CompositeRenderable(().into()))
            .with(CompositeRenderDepth(-100.0))
            .with(CompositeTransform::default())
            .with(CompositeMapChunk::new("map.map".into(), "roads".into()))
            .with(NonPersistent(token))
            .build();

        let (hw, hh) = {
            let assets = world.read_resource::<AssetsDatabase>();
            let asset = assets.asset_by_path("map://map.map").unwrap();
            let map = asset.get::<MapAsset>().unwrap().map();
            let (w, h) = map.size();
            world.write_resource::<Globals>().map_size = Some([w as f32, h as f32].into());
            (w as f64 * 0.5, h as f64 * 0.5)
        };

        world
            .create_entity()
            .with(RigidBody2d::new(
                RigidBodyDesc::new()
                    .translation(Vector::new(hw, -HALF_WALL_THICKNESS))
                    .gravity_enabled(false),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(hw, HALF_WALL_THICKNESS)),
            ))))
            .with(Collider2dBody::Me)
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(RigidBody2d::new(
                RigidBodyDesc::new()
                    .translation(Vector::new(hw, hh + hh + HALF_WALL_THICKNESS))
                    .gravity_enabled(false),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(hw, HALF_WALL_THICKNESS)),
            ))))
            .with(Collider2dBody::Me)
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(RigidBody2d::new(
                RigidBodyDesc::new()
                    .translation(Vector::new(-HALF_WALL_THICKNESS, hh))
                    .gravity_enabled(false),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(HALF_WALL_THICKNESS, hh)),
            ))))
            .with(Collider2dBody::Me)
            .with(NonPersistent(token))
            .build();

        world
            .create_entity()
            .with(RigidBody2d::new(
                RigidBodyDesc::new()
                    .translation(Vector::new(hw + hw + HALF_WALL_THICKNESS, hh))
                    .gravity_enabled(false),
            ))
            .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                Cuboid::new(Vector::new(HALF_WALL_THICKNESS, hh)),
            ))))
            .with(Collider2dBody::Me)
            .with(NonPersistent(token))
            .build();
    }

    fn create_trees(world: &mut World, token: StateToken) {
        let mut rng = thread_rng();

        let trees_meta = {
            let assets = world.read_resource::<AssetsDatabase>();
            let asset = assets.asset_by_path("map://map.map").unwrap();
            let map = asset.get::<MapAsset>().unwrap().map();
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
                                    let t = match spawn.name.as_str() {
                                        "barricade" => t_range_sand.sample(&mut rng),
                                        "tree" => t_range_grass.sample(&mut rng),
                                        _ => unreachable!(),
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
            let (tree_type, depth, radius) = match t {
                0 => ("treeGreen_large.png", 100.0, 32.0),
                1 => ("treeGreen_small.png", 50.0, 12.0),
                2 => ("treeBrown_large.png", 100.0, 32.0),
                3 => ("treeBrown_small.png", 50.0, 12.0),
                4 => ("barricadeMetal.png", 10.0, 24.0),
                5 => ("barricadeWood.png", 10.0, 24.0),
                _ => unreachable!(),
            };
            world
                .create_entity()
                .with(Tag("world".into()))
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
                        .gravity_enabled(false)
                        .rotation(r),
                ))
                .with(Collider2d::new(ColliderDesc::new(ShapeHandle::new(
                    Ball::new(radius),
                ))))
                .with(Collider2dBody::Me)
                .with(Physics2dSyncCompositeTransform)
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
                            let (r, t) = match spawn.name.as_str() {
                                "north" => (0.0, PlayerType::North),
                                "south" => (PI, PlayerType::South),
                                _ => unreachable!(),
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
            let entity = world
                .create_entity()
                .with(Tag("world".into()))
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
                        .gravity_enabled(false)
                        .rotation(r)
                        .linear_damping(0.5)
                        .angular_damping(2.0),
                ))
                .with(Collider2d::new(
                    ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector::new(40.0, 40.0))))
                        .density(1.0),
                ))
                .with(Collider2dBody::Me)
                .with(Physics2dSyncCompositeTransform)
                .with(NonPersistent(token))
                .with(Speed(50.0, 0.1))
                .with(Player(t))
                .with(Health(3))
                .with(Ammo(2))
                .with(Death(DespawnEffect::Explode))
                .build();
            world.write_resource::<TurnManager>().register(entity);
        }
    }

    fn create_bonuses(world: &mut World, token: StateToken) {
        let mut rng = thread_rng();

        let bonuses_meta = {
            let assets = world.read_resource::<AssetsDatabase>();
            let asset = assets.asset_by_path("map://map.map").unwrap();
            let map = asset.get::<MapAsset>().unwrap().map();
            let spawns = map.layer_by_name("spawns").unwrap().data.objects().unwrap();
            spawns
                .into_iter()
                .flat_map(|spawn| {
                    if !spawn.visible {
                        return vec![];
                    }
                    match spawn.object_type.as_str() {
                        "spawn" => {
                            let x_range = Uniform::from(spawn.x..(spawn.x + spawn.width as isize));
                            let y_range = Uniform::from(spawn.y..(spawn.y + spawn.height as isize));
                            let r_range = Uniform::from(0.0..PI);
                            let count = match spawn.name.as_str() {
                                "bonuses" => rng.next_u32() % 2 + 1,
                                "barrels" => 4,
                                _ => unreachable!(),
                            };
                            (0..count)
                                .map(|_| {
                                    let x = x_range.sample(&mut rng);
                                    let y = y_range.sample(&mut rng);
                                    let r = r_range.sample(&mut rng);
                                    let t = match spawn.name.as_str() {
                                        "bonuses" => rng.next_u32() % 2,
                                        "barrels" => rng.next_u32() % 2 + 2,
                                        _ => unreachable!(),
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

        for (x, y, r, t) in bonuses_meta {
            let bonus_type = match t {
                0 => "sandbagBeige.png",
                1 => "sandbagBrown.png",
                2 => "barrelRed_side.png",
                3 => "barrelRed_top.png",
                _ => unreachable!(),
            };
            let mut entity_builder = world
                .create_entity()
                .with(Tag("world".into()))
                .with(CompositeRenderable(().into()))
                .with(CompositeRenderDepth(1.0))
                .with(
                    CompositeSprite::new("sprites.0.json".into(), bonus_type.into())
                        .align(0.5.into()),
                )
                .with(CompositeTransform::default())
                .with(RigidBody2d::new(
                    RigidBodyDesc::new()
                        .translation(Vector::new(x as f64, y as f64))
                        .gravity_enabled(false)
                        .rotation(r)
                        .linear_damping(0.5)
                        .angular_damping(2.0),
                ))
                .with(Collider2d::new(
                    ColliderDesc::new(ShapeHandle::new(Ball::new(32.0))).density(1.0),
                ))
                .with(Collider2dBody::Me)
                .with(Physics2dSyncCompositeTransform)
                .with(NonPersistent(token));
            if t == 0 {
                entity_builder = entity_builder.with(Bonus(BonusType::Ammo(1)));
            } else if t == 1 {
                entity_builder = entity_builder.with(Bonus(BonusType::Health(1)));
            } else {
                entity_builder = entity_builder.with(Health(2));
            }
            entity_builder.build();
        }
    }

    fn create_ui(&mut self, world: &mut World, token: StateToken) {
        self.ui_timer = Some(
            world
                .create_entity()
                .with(Tag("ui".into()))
                .with(CompositeRenderable(
                    Text::new("Verdana".into(), "".into())
                        .color(Color::yellow())
                        .size(48.0)
                        .into(),
                ))
                .with(CompositeTransform::translation([24.0, 48.0].into()))
                .with(CompositeCameraAlignment(0.0.into()))
                .with(NonPersistent(token))
                .build(),
        );

        self.ui_player = Some(
            world
                .create_entity()
                .with(Tag("ui".into()))
                .with(CompositeRenderable(
                    Text::new("Verdana".into(), "".into())
                        .color(Color::yellow())
                        .align(TextAlign::Right)
                        .size(48.0)
                        .into(),
                ))
                .with(CompositeTransform::translation([-24.0, 48.0].into()))
                .with(CompositeCameraAlignment([1.0, 0.0].into()))
                .with(NonPersistent(token))
                .build(),
        );

        self.ui_panel = Some(
            world
                .create_entity()
                .with(Tag("ui".into()))
                .with(CompositeRenderable(
                    Rectangle {
                        color: Color::black().a(192),
                        rect: Rect::with_size([600.0, 480.0].into()).align(0.5.into()),
                    }
                    .into(),
                ))
                .with(CompositeTransform::default())
                .with(CompositeVisibility(true))
                .with(NonPersistent(token))
                .build(),
        );

        self.ui_panel_text = Some(
            world
                .create_entity()
                .with(Tag("ui".into()))
                .with(CompositeRenderable(
                    Text::new("Verdana".into(), "".into())
                        .color(Color::yellow())
                        .align(TextAlign::Center)
                        .baseline(TextBaseLine::Middle)
                        .size(48.0)
                        .into(),
                ))
                .with(CompositeTransform::default())
                .with(CompositeVisibility(true))
                .with(NonPersistent(token))
                .build(),
        );
    }

    // TODO: move to UiUpdateSystem?
    fn update_ui(&self, world: &mut World) {
        if let Some(ui_timer) = self.ui_timer {
            let timer = world.read_resource::<TurnManager>().timer();
            if let Some(renderable) = world
                .write_storage::<CompositeRenderable>()
                .get_mut(ui_timer)
            {
                if let Renderable::Text(text) = &mut renderable.0 {
                    text.text = match timer {
                        Timer::Waiting(t) => format!("Waiting: {}s", t as isize + 1).into(),
                        Timer::Playing(t) => format!("Playing: {}s", t as isize + 1).into(),
                        _ => "".into(),
                    };
                }
            }
        }
        if let Some(ui_player) = self.ui_player {
            let player_type_and_health = if let Some(selected) =
                world.read_resource::<TurnManager>().selected()
            {
                let player = world.read_storage::<Player>().get(selected).cloned();
                let health = world.read_storage::<Health>().get(selected).cloned();
                let ammo = world.read_storage::<Ammo>().get(selected).cloned();
                match (player, health, ammo) {
                    (Some(player), Some(health), Some(ammo)) => Some((player.0, health.0, ammo.0)),
                    _ => None,
                }
            } else {
                None
            };
            if let Some((player_type, health, ammo)) = player_type_and_health {
                if let Some(renderable) = world
                    .write_storage::<CompositeRenderable>()
                    .get_mut(ui_player)
                {
                    if let Renderable::Text(text) = &mut renderable.0 {
                        text.text = match player_type {
                            PlayerType::North => {
                                format!("North\nHealth: {}\nAmmo: {}", health, ammo).into()
                            }
                            PlayerType::South => {
                                format!("South\nHealth: {}\nAmmo: {}", health, ammo).into()
                            }
                        };
                    }
                }
            }
        }

        // TODO: before doing something check if game phase has changed to avoid allocations.
        let phase = world.read_resource::<Globals>().phase;
        if let Some(ui_panel) = self.ui_panel {
            if let Some(visibility) = world
                .write_storage::<CompositeVisibility>()
                .get_mut(ui_panel)
            {
                visibility.0 = !phase.is_game();
            }
        }
        if let Some(ui_panel_text) = self.ui_panel_text {
            if let Some(visibility) = world
                .write_storage::<CompositeVisibility>()
                .get_mut(ui_panel_text)
            {
                visibility.0 = !phase.is_game();
            }
            if let Some(renderable) = world
                .write_storage::<CompositeRenderable>()
                .get_mut(ui_panel_text)
            {
                if let Renderable::Text(text) = &mut renderable.0 {
                    text.text = match phase {
                        GamePhase::Start => {
                            "Hotseat fight between\ntwo players.\n\nWSAD to move\nSPACE to shoot"
                                .into()
                        }
                        GamePhase::End(None) => "Opps, looks like\nno one won!".into(),
                        GamePhase::End(Some(player_type)) => {
                            format!("{:?} player won!", player_type).into()
                        }
                        _ => "".into(),
                    };
                }
            }
        }
    }

    // TODO: move to SpawnSystem
    fn spawn_and_despawn(world: &mut World) {
        let token = world.read_resource::<AppLifeCycle>().current_state_token();
        let spawns = world.write_resource::<Spawner>().take_spawns();
        for spawn in spawns {
            match spawn {
                Spawn::Bullet(p, r, t, v) => {
                    let bullet_type = match t {
                        PlayerType::North => "bulletRed3_outline.png",
                        PlayerType::South => "bulletBlue3_outline.png",
                    };
                    world
                        .create_entity()
                        .with(Tag("world".into()))
                        .with(CompositeRenderable(().into()))
                        .with(CompositeRenderDepth(30.0))
                        .with(
                            CompositeSprite::new("sprites.0.json".into(), bullet_type.into())
                                .align(0.5.into()),
                        )
                        .with(CompositeTransform::default())
                        .with(RigidBody2d::new(
                            RigidBodyDesc::new()
                                .translation(p)
                                .gravity_enabled(false)
                                .rotation(r + PI)
                                .velocity(v),
                        ))
                        .with(Collider2d::new(
                            ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector::new(
                                8.0, 18.0,
                            ))))
                            .density(1.0),
                        ))
                        .with(Collider2dBody::Me)
                        .with(Physics2dSyncCompositeTransform)
                        .with(NonPersistent(token))
                        .with(Bullet(t))
                        .with(Death(DespawnEffect::ExplodeSmoke))
                        .with(Lifetime(3.0))
                        .with(AudioSource::from(
                            AudioSourceConfig::new("sounds/firegun.ogg".into()).play(true),
                        ))
                        .build();
                }
            }
        }
        let despawns = world.write_resource::<Spawner>().take_despawns();
        for (entity, effect) in despawns {
            match effect {
                DespawnEffect::Explode | DespawnEffect::ExplodeSmoke => {
                    let pos = world
                        .read_storage::<CompositeTransform>()
                        .get(entity)
                        .map(|t| t.get_translation());
                    if let Some(pos) = pos {
                        let animations = match effect {
                            DespawnEffect::Explode => {
                                let mut animations = HashMap::with_capacity(1);
                                animations.insert(
                                    "explode".into(),
                                    SpriteAnimation {
                                        sheet: "sprites.0.json".into(),
                                        frames: vec![
                                            "explosion1.png".into(),
                                            "explosion2.png".into(),
                                            "explosion3.png".into(),
                                            "explosion4.png".into(),
                                            "explosion5.png".into(),
                                        ],
                                    },
                                );
                                animations
                            }
                            DespawnEffect::ExplodeSmoke => {
                                let mut animations = HashMap::with_capacity(1);
                                animations.insert(
                                    "explode".into(),
                                    SpriteAnimation {
                                        sheet: "sprites.0.json".into(),
                                        frames: vec![
                                            "explosionSmoke1.png".into(),
                                            "explosionSmoke2.png".into(),
                                            "explosionSmoke3.png".into(),
                                            "explosionSmoke4.png".into(),
                                            "explosionSmoke5.png".into(),
                                        ],
                                    },
                                );
                                animations
                            }
                            _ => unreachable!(),
                        };
                        world
                            .create_entity()
                            .with(Tag("world".into()))
                            .with(CompositeRenderable(().into()))
                            .with(CompositeRenderDepth(200.0))
                            .with(CompositeSprite::default().align(0.5.into()))
                            .with(CompositeSpriteAnimation::new(animations).autoplay(
                                "explode",
                                5.0 / EXPLOSION_TIME as f32,
                                false,
                            ))
                            .with(CompositeTransform::translation(pos))
                            .with(NonPersistent(token))
                            .with(Lifetime(EXPLOSION_TIME))
                            .with(AudioSource::from(
                                AudioSourceConfig::new("sounds/explosion.ogg".into()).play(true),
                            ))
                            .build();
                    }
                }
                _ => {}
            }
            drop(world.delete_entity(entity));
            world.write_resource::<TurnManager>().unregister(entity);
        }
    }
}
