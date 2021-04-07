use crate::ui::screens::{menu::hook::*, notifications::*};
use oxygengine::{prelude::*, user_interface::raui::core::widget::WidgetId};

#[derive(Debug, Default)]
pub struct GameState {
    camera: Option<Entity>,
    player: Option<Entity>,
    menu: Option<WidgetId>,
    notifications: Option<WidgetId>,
    change: Scalar,
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // instantiate world objects from scene prefab.
        let camera = world
            .write_resource::<PrefabManager>()
            .instantiate_world("new-bark-town", world)
            .unwrap()[0];
        self.camera = Some(camera);

        // instantiate player from prefab.
        let player = world
            .write_resource::<PrefabManager>()
            .instantiate_world("player", world)
            .unwrap()[0];
        self.player = Some(player);

        // setup created player instance.
        world.read_resource::<LazyUpdate>().exec(move |world| {
            let mut transform = <CompositeTransform>::fetch(world, player);
            let pos = Vec2::new(16.0 * 12.0, 16.0 * 11.0);
            transform.set_translation(pos);
        });
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let (Some(player), Some(camera)) = (self.player, self.camera) {
            let mut transforms = world.write_storage::<CompositeTransform>();
            // NOTE: REMEMBER THAT PREFABS ARE INSTANTIATED IN NEXT FRAME, SO THEY MIGHT NOT EXIST
            // AT FIRST SO HANDLE THAT.
            if let Some(player_transform) = transforms.get(player) {
                let player_position = player_transform.get_translation();
                if let Some(camera_transform) = transforms.get_mut(camera) {
                    camera_transform.set_translation(player_position);
                }
            }
        }

        let mut ui = world.write_resource::<UserInterfaceRes>();
        if let Some(app) = ui.application_mut("") {
            for (caller, msg) in app.consume_signals() {
                if let Some(msg) = msg.as_any().downcast_ref::<NotificationSignal>() {
                    match msg {
                        NotificationSignal::Register => self.notifications = Some(caller),
                        NotificationSignal::Unregister => {
                            if let Some(id) = &self.notifications {
                                if &caller == id {
                                    self.notifications = None;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(msg) = msg.as_any().downcast_ref::<MenuSignal>() {
                    match msg {
                        MenuSignal::Register => self.menu = Some(caller),
                        MenuSignal::Unregister => {
                            if let Some(id) = &self.menu {
                                if &caller == id {
                                    self.menu = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            let input = world.read_resource::<InputController>();
            if input.trigger_or_default("escape") == TriggerState::Pressed {
                if let Some(id) = &self.menu {
                    app.send_message(id, MenuSignal::Show);
                }
            }

            self.change -= world.read_resource::<AppLifeCycle>().delta_time_seconds();
            if self.change <= 0.0 {
                self.change = 2.0 + rand::random::<Scalar>() * 10.0;
                if let Some(id) = &self.notifications {
                    if rand::random() {
                        app.send_message(
                            id,
                            NotificationSignal::Show(NotificationShow {
                                text: "There is a fog somewhere in Wild Area".to_owned(),
                                ..Default::default()
                            }),
                        );
                    } else {
                        app.send_message(
                            id,
                            NotificationSignal::Show(NotificationShow {
                                text: "Thanks for using RAUI".to_owned(),
                                side: true,
                                ..Default::default()
                            }),
                        );
                    }
                }
            }
        }

        StateChange::None
    }
}
