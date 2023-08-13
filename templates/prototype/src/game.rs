use crate::nodes::{character::*, indicator::*};
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut nodes = universe.expect_resource_mut::<ScriptedNodes>();

        nodes.spawn(
            ScriptedNodesTree::new(
                Character::new(crate::assets::image::CRAB)
                    .size(100.0)
                    .speed(250.0)
                    .animation_speed(15.0),
            )
            .component(HaTransform::default())
            .child(|_| {
                ScriptedNodesTree::new(
                    Indicator::new(crate::assets::image::LOGO)
                        .size(50.0)
                        .animation_speed(5.0),
                )
                .component(HaTransform::translation(vec3(0.0, -70.0, 0.0)))
            }),
            None,
        );
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut renderables = universe.expect_resource_mut::<Renderables>();
        let lifecycle = universe.expect_resource::<AppLifeCycle>();
        let inputs = universe.expect_resource::<InputController>();
        let mut audio = universe.expect_resource_mut::<AudioPlayer>();
        let camera = universe.expect_resource::<Camera>();
        let mut nodes = universe.expect_resource_mut::<ScriptedNodes>();

        let dt = lifecycle.delta_time_seconds();
        let clicked = inputs.trigger_or_default("mouse-action").is_pressed();
        let pointer = Vec2::from(inputs.multi_axis_or_default(["mouse-x", "mouse-y"]));
        if clicked {
            audio.play(crate::assets::audio::POP, 1.0);
        }

        nodes.maintain(universe);
        nodes.dispatch::<&mut HaTransform>(
            &universe,
            ScriptFunctionReference::parse("event_update").unwrap(),
            &[(&dt).into(), (&*inputs).into()],
        );
        nodes.dispatch::<&mut HaTransform>(
            &universe,
            ScriptFunctionReference::parse("event_draw").unwrap(),
            &[(&dt).into(), (&mut *renderables).into()],
        );

        gui(pointer, &camera, &mut renderables, &mut (), |mut gui| {
            gui.margin(16.0, 16.0, 16.0, 16.0, |mut gui| {
                gui.cut_bottom(75.0, |mut gui| {
                    message_panel_widget(
                        gui.gui(),
                        crate::assets::image::LOGO,
                        "Welcome to Oxygengine's prototyping module, let's have some fun!",
                        "Use WSAD keys to move Ferris around!",
                    );
                });
            });
        });

        StateChange::None
    }
}

fn message_panel_widget<T>(
    mut gui: Gui<T>,
    avatar: &str,
    message_normal: &str,
    message_hovers: &str,
) {
    gui.button(move |mut gui, hovers| {
        gui.sprite_sliced(
            crate::assets::image::PANEL,
            Rgba::white(),
            rect(0.4, 0.4, 0.2, 0.2),
            (16.0, 16.0, 16.0, 16.0),
            false,
        );
        gui.cut_left(gui.layout().h, |mut gui| {
            avatar_widget(gui.gui(), avatar, hovers);
        });
        gui.text(
            crate::assets::font::ROBOTO,
            if hovers {
                message_hovers
            } else {
                message_normal
            },
            20.0,
            Rgba::white(),
            0.0,
        );
        false
    });
}

fn avatar_widget<T>(mut gui: Gui<T>, image: &str, hovers: bool) {
    if hovers {
        gui.clip(|mut gui| {
            gui.scale(1.25, 0.5, |mut gui| {
                gui.sprite(image, Rgba::white());
            });
        });
    } else {
        gui.sprite(image, Rgba::white());
    };
}
