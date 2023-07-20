use crate::character::*;
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    crab: Option<Character>,
}

impl State for GameState {
    fn on_enter(&mut self, _: &mut Universe) {
        self.crab = Some(
            Character::new(crate::assets::image::CRAB)
                .size(100.0)
                .speed(100.0)
                .animation_speed(10.0),
        );
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut renderables = universe.expect_resource_mut::<Renderables>();
        let lifecycle = universe.expect_resource::<AppLifeCycle>();
        let inputs = universe.expect_resource::<InputController>();
        let mut audio = universe.expect_resource_mut::<AudioPlayer>();
        let camera = universe.expect_resource::<Camera>();

        let dt = lifecycle.delta_time_seconds();
        let clicked = inputs.trigger_or_default("mouse-action").is_pressed();
        let pointer = Vec2::from(inputs.multi_axis_or_default(["mouse-x", "mouse-y"]));
        let player_move =
            Vec2::from(inputs.mirror_multi_axis_or_default([
                ("move-right", "move-left"),
                ("move-down", "move-up"),
            ]));

        if clicked {
            audio.play(crate::assets::audio::POP, 1.0);
        }

        if let Some(crab) = self.crab.as_mut() {
            crab.move_position(dt, player_move);
            crab.update_animation(dt);
            crab.draw(&mut renderables);
        }

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
