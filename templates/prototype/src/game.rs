use crate::nodes::{character::*, enemy::*, gui::*, thunder::*};
use oxygengine::prelude::*;
use rand::{thread_rng, Rng};
use std::f32::consts::TAU;

#[derive(Debug)]
enum Phase {
    Start,
    Play,
    GameOver,
}

#[derive(Debug)]
pub struct GameState {
    spawn_interval: Scalar,
    spawn_accumulator: Scalar,
    spawn_distance: Scalar,
    phase: Phase,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            spawn_interval: 0.5,
            spawn_accumulator: 0.0,
            spawn_distance: 400.0,
            phase: Phase::Start,
        }
    }
}

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut spawns = universe.expect_resource_mut::<ScriptedNodesSpawns>();

        spawns.spawn_root(
            ScriptedNodesTree::new(SpriteNode {
                image: crate::assets::image::GRASS.to_owned(),
                tiling: 8.0.into(),
                size: 1024.0.into(),
                ..Default::default()
            })
            .component(HaTransform::default()),
        );
        spawns.spawn_root(Self::create_player(vec2(0.0, 0.0)));
        spawns.spawn_root(
            ScriptedNodesTree::new(GuiNode::default()).component(HaTransform::default()),
        );
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        use oxygengine::prototype::nodes::*;

        let dt = universe
            .expect_resource::<AppLifeCycle>()
            .delta_time_seconds();
        let player_exists =
            ScriptedNodeEntity::find("player", &universe.expect_resource::<Hierarchy>()).is_valid();

        match self.phase {
            Phase::Start => {
                if player_exists {
                    self.phase = Phase::Play;
                    universe
                        .expect_resource_mut::<ScriptedNodesSignals>()
                        .signal::<()>(
                            ScriptedNodeSignal::parse(None, "signal_game_start")
                                .unwrap()
                                .broadcast(),
                        );
                }
            }
            Phase::Play => {
                self.spawn_accumulator -= dt;
                if self.spawn_accumulator <= 0.0 {
                    let (y, x) = thread_rng().gen_range(0.0..TAU).sin_cos();
                    universe
                        .expect_resource_mut::<ScriptedNodesSpawns>()
                        .spawn_root(Self::create_enemy(vec2(x, y) * self.spawn_distance));
                    self.spawn_accumulator = self.spawn_interval;
                }

                if !player_exists {
                    self.phase = Phase::GameOver;
                    universe
                        .expect_resource_mut::<ScriptedNodesSignals>()
                        .signal::<()>(
                            ScriptedNodeSignal::parse(None, "signal_game_over")
                                .unwrap()
                                .broadcast(),
                        );
                }
            }
            Phase::GameOver => {
                if player_exists {
                    self.phase = Phase::Start;
                }
            }
        }

        universe.expect_resource_mut::<SpatialQueries>().clear();
        ScriptedNodes::maintain(universe);
        dispatch_events(
            universe,
            &[EVENT_PREPARE, EVENT_UPDATE, EVENT_DRAW, EVENT_DRAW_GUI],
        );

        StateChange::None
    }
}

impl GameState {
    pub fn create_player(position: Vec2) -> ScriptedNodesTree {
        let mut this = ScriptedNodeEntity::default();
        let this2 = this.clone();
        let mut sprite = ScriptedNodeEntity::default();
        let mut thunder = ScriptedNodeEntity::default();

        ScriptedNodesTree::new(CharacterNode {
            speed: 200.0,
            animation_speed: 20.0,
            sprite: sprite.clone(),
            thunder: thunder.clone(),
            ..Default::default()
        })
        .name("player")
        .component(HaTransform::translation(position.into()))
        .bind(move |entity| this.set(entity))
        .child(move || {
            ScriptedNodesTree::new(SpatialNode {
                area: Rect::new(-40.0, -40.0, 80.0, 80.0),
                entity: this2,
            })
            .component(HaTransform::default())
        })
        .child(move || {
            ScriptedNodesTree::new(SpriteNode {
                image: crate::assets::image::CRAB.to_owned(),
                size: 100.0.into(),
                region: SpriteNodeRegion::AnimationFrame {
                    frame: 0,
                    cols: 4,
                    rows: 1,
                },
                ..Default::default()
            })
            .component(HaTransform::default())
            .bind(move |entity| sprite.set(entity))
        })
        .child(move || {
            ScriptedNodesTree::new(ThunderNode {
                size: 3.0,
                spread: 25.0,
                ..Default::default()
            })
            .component(HaTransform::default())
            .bind(move |entity| thunder.set(entity))
        })
    }

    pub fn create_enemy(position: Vec2) -> ScriptedNodesTree {
        let mut this = ScriptedNodeEntity::default();
        let this2 = this.clone();
        let mut sprite = ScriptedNodeEntity::default();
        let mut spatial = ScriptedNodeEntity::default();

        ScriptedNodesTree::new(EnemyNode {
            speed: 50.0,
            animation_speed: 8.0,
            sprite: sprite.clone(),
            spatial: spatial.clone(),
            ..Default::default()
        })
        .component(HaTransform::translation(position.into()))
        .bind(move |entity| this.set(entity))
        .child(move || {
            ScriptedNodesTree::new(SpatialNode {
                area: Rect::new(-40.0, -40.0, 80.0, 80.0),
                entity: this2,
            })
            .component(HaTransform::default())
            .bind(move |entity| spatial.set(entity))
        })
        .child(move || {
            ScriptedNodesTree::new(SpriteNode {
                image: crate::assets::image::GOPHER.to_owned(),
                size: 100.0.into(),
                region: SpriteNodeRegion::AnimationFrame {
                    frame: 0,
                    cols: 4,
                    rows: 1,
                },
                ..Default::default()
            })
            .component(HaTransform::default())
            .bind(move |entity| sprite.set(entity))
        })
    }
}
