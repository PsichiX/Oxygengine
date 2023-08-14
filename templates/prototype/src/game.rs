use crate::nodes::{character::*, grass::*, gui::*, indicator::*};
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut spawns = universe.expect_resource_mut::<ScriptedNodesSpawns>();
        let mut indicator = ScriptedNodeEntity::default();

        spawns.spawn_root(
            ScriptedNodesTree::new(
                Grass::new(crate::assets::image::GRASS)
                    .size(1024.0)
                    .tiling(8.0),
            )
            .component(HaTransform::default())
            .child(move |_| {
                ScriptedNodesTree::new(
                    Character::new(crate::assets::image::CRAB, indicator.clone())
                        .size(100.0)
                        .speed(250.0)
                        .animation_speed(15.0),
                )
                .component(HaTransform::default())
                .child(move |_| {
                    ScriptedNodesTree::new(
                        Indicator::new(crate::assets::image::LOGO)
                            .size(50.0)
                            .animation_speed(5.0),
                    )
                    .component(HaTransform::translation(vec3(0.0, -70.0, 0.0)))
                    .bind(move |entity| indicator.set(entity))
                })
            })
            .child(|_| ScriptedNodesTree::new(GameUi).component(HaTransform::default())),
        );
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        ScriptedNodes::maintain(universe);

        let lifecycle = &*universe.expect_resource::<AppLifeCycle>();
        let nodes = &mut *universe.expect_resource_mut::<ScriptedNodes>();
        let signals = &mut *universe.expect_resource_mut::<ScriptedNodesSignals>();
        let renderables = &mut *universe.expect_resource_mut::<Renderables>();
        let inputs = &*universe.expect_resource::<InputController>();
        let camera = &*universe.expect_resource::<Camera>();
        let dt = lifecycle.delta_time_seconds();

        nodes.dispatch::<&mut HaTransform>(
            universe,
            ScriptFunctionReference::parse("event_update").unwrap(),
            &[(&dt).into(), inputs.into(), signals.into()],
        );
        nodes.dispatch::<&mut HaTransform>(
            universe,
            ScriptFunctionReference::parse("event_draw").unwrap(),
            &[(&dt).into(), renderables.into()],
        );
        nodes.dispatch::<&mut HaTransform>(
            universe,
            ScriptFunctionReference::parse("event_draw_gui").unwrap(),
            &[camera.into(), inputs.into(), renderables.into()],
        );

        // TODO: move to node.
        let mut audio = universe.expect_resource_mut::<AudioPlayer>();
        let clicked = inputs.trigger_or_default("mouse-action").is_pressed();
        if clicked {
            audio.play(crate::assets::audio::POP, 1.0);
        }

        StateChange::None
    }
}
