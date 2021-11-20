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
    fn on_enter(&mut self, universe: &mut Universe) {
        // instantiate world objects from scene prefab.
        let camera = universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("new-bark-town", universe)
            .unwrap()[0];
        self.camera = Some(camera);

        // instantiate player from prefab.
        let player = universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("player", universe)
            .unwrap()[0];
        self.player = Some(player);

        // setup created player instance.
        if let Ok(mut transform) = universe
            .world()
            .query_one::<&mut CompositeTransform>(player)
        {
            if let Some(transform) = transform.get() {
                transform.set_translation(Vec2::new(16.0 * 12.0, 16.0 * 11.0));
            }
        }
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        if let (Some(player), Some(camera)) = (self.player, self.camera) {
            let player_position = match universe.world().query_one::<&CompositeTransform>(player) {
                Ok(mut transform) => match transform.get() {
                    Some(transform) => transform.get_translation(),
                    _ => Default::default(),
                },
                _ => Default::default(),
            };
            if let Ok(mut transform) = universe
                .world()
                .query_one::<&mut CompositeTransform>(camera)
            {
                if let Some(transform) = transform.get() {
                    transform.set_translation(player_position);
                }
            }
        }

        let mut ui = universe.expect_resource_mut::<UserInterface>();
        if let Some(data) = ui.get_mut("") {
            for (caller, msg) in data.signals_received() {
                if let Some(msg) = msg.as_any().downcast_ref::<NotificationSignal>() {
                    match msg {
                        NotificationSignal::Register => {
                            self.notifications = Some(caller.to_owned())
                        }
                        NotificationSignal::Unregister => {
                            if let Some(id) = &self.notifications {
                                if caller == id {
                                    self.notifications = None;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(msg) = msg.as_any().downcast_ref::<MenuSignal>() {
                    match msg {
                        MenuSignal::Register => self.menu = Some(caller.to_owned()),
                        MenuSignal::Unregister => {
                            if let Some(id) = &self.menu {
                                if caller == id {
                                    self.menu = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            let input = universe.expect_resource::<InputController>();
            if input.trigger_or_default("escape") == TriggerState::Pressed {
                if let Some(id) = &self.menu {
                    data.application.send_message(id, MenuSignal::Show);
                }
            }

            self.change -= universe
                .expect_resource::<AppLifeCycle>()
                .delta_time_seconds();
            if self.change <= 0.0 {
                self.change = 2.0 + rand::random::<Scalar>() * 10.0;
                if let Some(id) = &self.notifications {
                    if rand::random() {
                        data.application.send_message(
                            id,
                            NotificationSignal::Show(NotificationShow {
                                text: "There is a fog somewhere in Wild Area".to_owned(),
                                ..Default::default()
                            }),
                        );
                    } else {
                        data.application.send_message(
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
