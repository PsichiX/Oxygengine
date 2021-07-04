use crate::{
    component::UserInterfaceView,
    resource::{ApplicationData, UserInterface},
    ui_theme_asset_protocol::UiThemeAsset,
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, ResRead, ResWrite, Universe, UnsafeRef, UnsafeScope, WorldMut, WorldRef},
};
use input::resource::{InputController, TriggerState};
use raui_core::{
    application::{Application, ProcessContext},
    interactive::default_interactions_engine::{Interaction, PointerButton},
    layout::default_layout_engine::DefaultLayoutEngine,
    widget::{
        component::interactive::navigation::{NavSignal, NavTextChange},
        setup as core_setup,
        utils::Vec2,
    },
};
use raui_material::{setup as material_setup, theme::ThemeProps};
use std::collections::HashMap;

#[derive(Default)]
pub struct UserInterfaceSystemCache {
    pub process_context_setup: Option<Box<dyn Fn(&Universe, &mut ProcessContext) + Send + Sync>>,
    last_pointer_pos: Vec2,
    themes_cache: HashMap<String, ThemeProps>,
    themes_table: HashMap<AssetId, String>,
}

impl UserInterfaceSystemCache {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Universe, &mut ProcessContext) + Send + Sync + 'static,
    {
        Self {
            process_context_setup: Some(Box::new(f)),
            last_pointer_pos: Default::default(),
            themes_cache: Default::default(),
            themes_table: Default::default(),
        }
    }

    pub fn theme(&self, id: &str) -> Option<&ThemeProps> {
        self.themes_cache.get(id)
    }
}

pub type UserInterfaceSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a mut UserInterface,
    &'a InputController,
    &'a mut UserInterfaceSystemCache,
    Comp<&'a mut UserInterfaceView>,
);

pub fn user_interface_system(universe: &mut Universe) {
    let mut cache = universe.expect_resource_mut::<UserInterfaceSystemCache>();
    {
        let assets = universe.expect_resource::<AssetsDatabase>();
        for id in assets.lately_loaded_protocol("ui-theme") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded UI theme asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<UiThemeAsset>()
                .expect("trying to use non UI theme asset");
            cache.themes_cache.insert(path.clone(), asset.get().props());
            cache.themes_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("ui-theme") {
            if let Some(path) = cache.themes_table.remove(id) {
                cache.themes_cache.remove(&path);
            }
        }
    }
    let scope = UnsafeScope;
    let mut ui = universe.expect_resource_mut::<UserInterface>();
    let meta = {
        let world = universe.world();
        let input = universe.expect_resource::<InputController>();

        ui.data = std::mem::take(&mut ui.data)
            .into_iter()
            .filter(|(k, _)| {
                world
                    .query::<&UserInterfaceView>()
                    .iter()
                    .any(|(_, v)| k == v.app_id())
            })
            .collect();

        for (_, view) in world.query::<&mut UserInterfaceView>().iter() {
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
                        signals_received: Default::default(),
                    },
                );
            }
            if view.dirty {
                view.dirty = false;
                let app = ui.application_mut(view.app_id()).unwrap();
                let mut root = app
                    .deserialize_node(view.root().clone())
                    .expect("Could not deserialize UI node");
                if let Some(theme) = view.theme() {
                    if let Some(theme) = cache.themes_cache.get(theme) {
                        if let Some(p) = root.shared_props_mut() {
                            p.write(theme.clone());
                        }
                    }
                }
                app.apply(root);
            }
        }

        let pointer_pos = Vec2 {
            x: input.axis_or_default(&ui.pointer_axis_x),
            y: input.axis_or_default(&ui.pointer_axis_y),
        };
        let pointer_moved = (pointer_pos.x - cache.last_pointer_pos.x).abs() > 1.0e-6
            || (pointer_pos.y - cache.last_pointer_pos.y).abs() > 1.0e-6;
        cache.last_pointer_pos = pointer_pos;
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

        let mut meta = world
            .query::<&UserInterfaceView>()
            .iter()
            .map(|(_, v)| unsafe { UnsafeRef::upgrade(&scope, v) })
            .collect::<Vec<_>>();
        meta.sort_by(|a, b| unsafe { a.read().input_order.cmp(&b.read().input_order) });
        meta
    };
    let dt = universe
        .expect_resource::<AppLifeCycle>()
        .delta_time_seconds();
    let mut captured = false;
    let mut context = ProcessContext::new();
    if let Some(f) = &cache.process_context_setup {
        f(universe, &mut context);
    }
    if context.has::<WorldMut>() {
        panic!("ProcessContext cannot contain WorldMut!");
    }
    if context.has::<ResRead<UserInterface>>() || context.has::<ResWrite<UserInterface>>() {
        panic!("ProcessContext cannot contain UserInterface!");
    }
    if context.has::<ResRead<UserInterfaceSystemCache>>()
        || context.has::<ResWrite<UserInterfaceSystemCache>>()
    {
        panic!("ProcessContext cannot contain UserInterfaceSystemCache!");
    }
    for view in meta {
        if let Some(data) = ui.data.get_mut(unsafe { view.read().app_id() }) {
            data.application.animations_delta_time = dt;
            data.application.process_with_context(&mut context);
            data.application
                .layout(&data.coords_mapping, &mut DefaultLayoutEngine)
                .unwrap_or_default();
            if captured {
                data.interactions.clear_queue(true);
            }
            data.interactions.deselect_when_no_button_found =
                unsafe { view.read().deselect_when_no_button_found };
            if let Ok(result) = data.application.interact(&mut data.interactions) {
                if unsafe { view.read().capture_input } && result.is_any() {
                    captured = true;
                }
            }
            data.signals_received = data.application.consume_signals();
        }
    }
    ui.last_frame_captured = captured;
}
