use crate::{
    component::UserInterfaceView,
    resource::{ApplicationData, UserInterfaceRes},
};
use core::{
    app::AppLifeCycle,
    ecs::{Join, Read, ReadExpect, System, Write, WriteStorage},
};
use input::resource::{InputController, TriggerState};
use raui_core::{
    application::Application,
    interactive::default_interactions_engine::{Interaction, PointerButton},
    layout::default_layout_engine::DefaultLayoutEngine,
    widget::{
        component::interactive::navigation::{NavSignal, NavTextChange},
        setup as core_setup,
        utils::Vec2,
    },
};
use raui_material::setup as material_setup;
use std::collections::HashMap;

#[derive(Default)]
pub struct UserInterfaceSystem {
    last_pointer_pos: Vec2,
}

impl<'s> System<'s> for UserInterfaceSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        Write<'s, UserInterfaceRes>,
        Read<'s, InputController>,
        WriteStorage<'s, UserInterfaceView>,
    );

    fn run(&mut self, (life_cycle, mut res, input, mut views): Self::SystemData) {
        let ui: &mut UserInterfaceRes = &mut *res;

        ui.data = std::mem::take(&mut ui.data)
            .into_iter()
            .filter(|(k, _)| views.join().any(|v| k == v.app_id()))
            .collect::<HashMap<_, _>>();
        for view in (&mut views).join() {
            if !ui.data.contains_key(view.app_id()) {
                let mut application = Application::new();
                application.setup(core_setup);
                application.setup(material_setup);
                if let Some(setup) = ui.setup {
                    setup(&mut application);
                }
                ui.data.insert(
                    view.app_id().to_owned(),
                    ApplicationData {
                        application,
                        interactions: Default::default(),
                        coords_mapping: Default::default(),
                    },
                );
            }
            if view.dirty {
                view.dirty = false;
                let app = ui.application_mut(view.app_id()).unwrap();
                let root = app
                    .deserialize_node(view.root().clone())
                    .expect("Could not deserialize UI node");
                app.apply(root);
            }
        }

        let pointer_pos = Vec2 {
            x: input.axis_or_default(&ui.pointer_axis_x),
            y: input.axis_or_default(&ui.pointer_axis_y),
        };
        let pointer_moved = (pointer_pos.x - self.last_pointer_pos.x).abs() > 1.0e-6
            || (pointer_pos.y - self.last_pointer_pos.y).abs() > 1.0e-6;
        self.last_pointer_pos = pointer_pos;
        let pointer_trigger = input.trigger_or_default(&ui.pointer_action_trigger);
        let pointer_context = input.trigger_or_default(&ui.pointer_context_trigger);
        let mut text = input
            .text()
            .chars()
            .filter_map(|c| {
                if !c.is_control() {
                    Some(NavTextChange::InsertCharacter(c))
                } else if c == '\n' || c == '\r' {
                    Some(NavTextChange::NewLine)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let accept = input.trigger_or_default(&ui.navigate_accept);
        let cancel = input.trigger_or_default(&ui.navigate_cancel);
        let up = input.trigger_or_default(&ui.navigate_up) == TriggerState::Pressed;
        let down = input.trigger_or_default(&ui.navigate_down) == TriggerState::Pressed;
        let left = input.trigger_or_default(&ui.navigate_left) == TriggerState::Pressed;
        let right = input.trigger_or_default(&ui.navigate_right) == TriggerState::Pressed;
        let prev = input.trigger_or_default(&ui.navigate_prev) == TriggerState::Pressed;
        let next = input.trigger_or_default(&ui.navigate_next) == TriggerState::Pressed;
        if input.trigger_or_default(&ui.text_move_cursor_left) == TriggerState::Pressed {
            text.push(NavTextChange::MoveCursorLeft);
        }
        if input.trigger_or_default(&ui.text_move_cursor_right) == TriggerState::Pressed {
            text.push(NavTextChange::MoveCursorRight);
        }
        if input.trigger_or_default(&ui.text_move_cursor_start) == TriggerState::Pressed {
            text.push(NavTextChange::MoveCursorStart);
        }
        if input.trigger_or_default(&ui.text_move_cursor_end) == TriggerState::Pressed {
            text.push(NavTextChange::MoveCursorEnd);
        }
        if input.trigger_or_default(&ui.text_delete_left) == TriggerState::Pressed {
            text.push(NavTextChange::DeleteLeft);
        }
        if input.trigger_or_default(&ui.text_delete_right) == TriggerState::Pressed {
            text.push(NavTextChange::DeleteRight);
        }
        for data in ui.data.values_mut() {
            let pointer_pos = data.coords_mapping.real_to_virtual_vec2(pointer_pos, false);
            if pointer_moved {
                data.interactions
                    .interact(Interaction::PointerMove(pointer_pos));
            }
            match pointer_trigger {
                TriggerState::Pressed => {
                    data.interactions.interact(Interaction::PointerDown(
                        PointerButton::Trigger,
                        pointer_pos,
                    ));
                }
                TriggerState::Released => {
                    data.interactions
                        .interact(Interaction::PointerUp(PointerButton::Trigger, pointer_pos));
                }
                _ => {}
            }
            match pointer_context {
                TriggerState::Pressed => {
                    data.interactions.interact(Interaction::PointerDown(
                        PointerButton::Context,
                        pointer_pos,
                    ));
                }
                TriggerState::Released => {
                    data.interactions
                        .interact(Interaction::PointerUp(PointerButton::Context, pointer_pos));
                }
                _ => {}
            }
            for change in &text {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::TextChange(*change)));
            }
            match accept {
                TriggerState::Pressed => {
                    data.interactions
                        .interact(Interaction::Navigate(NavSignal::Accept(true)));
                }
                TriggerState::Released => {
                    data.interactions
                        .interact(Interaction::Navigate(NavSignal::Accept(false)));
                }
                _ => {}
            }
            match cancel {
                TriggerState::Pressed => {
                    data.interactions
                        .interact(Interaction::Navigate(NavSignal::Cancel(true)));
                }
                TriggerState::Released => {
                    data.interactions
                        .interact(Interaction::Navigate(NavSignal::Cancel(false)));
                }
                _ => {}
            }
            if up {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Up));
            }
            if down {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Down));
            }
            if left {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Left));
            }
            if right {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Right));
            }
            if prev {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Prev));
            }
            if next {
                data.interactions
                    .interact(Interaction::Navigate(NavSignal::Next));
            }
        }

        let mut meta = views.join().collect::<Vec<_>>();
        meta.sort_by(|a, b| a.input_order.cmp(&b.input_order));
        let mut captured = false;
        for view in meta {
            if let Some(data) = ui.data.get_mut(view.app_id()) {
                data.application.animations_delta_time = life_cycle.delta_time_seconds();
                data.application.process();
                data.application
                    .layout(&data.coords_mapping, &mut DefaultLayoutEngine)
                    .unwrap_or_default();
                if captured {
                    data.interactions.clear_queue(true);
                }
                data.interactions.deselect_when_no_button_found =
                    view.deselect_when_no_button_found;
                if let Ok(result) = data.application.interact(&mut data.interactions) {
                    if view.capture_input && result.is_any() {
                        captured = true;
                    }
                }
            }
        }
        res.last_frame_captured = captured;
    }
}
