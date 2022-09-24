use crate::{
    component::UserInterfaceView,
    resource::{input_mappings::*, ApplicationData, UserInterface},
    ui_theme_asset_protocol::UiThemeAsset,
    FeedProcessContext,
};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{AccessType, Comp, ResQuery, ResQueryItem, Universe, UnsafeRef, UnsafeScope, WorldRef},
};
use input::{
    component::InputStackInstance,
    resources::stack::{InputStack, InputStackListener},
};
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
    themes_cache: HashMap<String, ThemeProps>,
    themes_table: HashMap<AssetId, String>,
}

impl UserInterfaceSystemCache {
    pub fn theme(&self, id: &str) -> Option<&ThemeProps> {
        self.themes_cache.get(id)
    }
}

pub type UserInterfaceSystemResources<'a, Q> = (
    Q,
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a InputStack,
    &'a mut UserInterface,
    &'a mut UserInterfaceSystemCache,
    Comp<&'a mut UserInterfaceView>,
    Comp<&'a InputStackInstance>,
);

pub fn user_interface_system<Q>(universe: &mut Universe)
where
    Q: AccessType + ResQuery + 'static,
    ResQueryItem<Q>: FeedProcessContext,
{
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

    let mut ui = universe.expect_resource_mut::<UserInterface>();
    let scope = UnsafeScope;
    let meta = {
        let world = universe.world();
        let input_stack = universe.expect_resource::<InputStack>();

        ui.data.retain(|k, _| {
            world
                .query::<&UserInterfaceView>()
                .iter()
                .any(|(_, v)| k == v.app_id())
        });

        for (_, (view, input)) in world
            .query::<(&mut UserInterfaceView, Option<&InputStackInstance>)>()
            .iter()
        {
            if !ui.data.contains_key(view.app_id()) {
                let mut application = Application::new();
                application.setup(core_setup);
                application.setup(material_setup);
                if let Some(setup_application) = ui.setup_application {
                    setup_application(&mut application);
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

            if let Some(listener) = input
                .and_then(|input| input.as_listener())
                .and_then(|id| input_stack.listener(id))
            {
                apply_inputs(
                    ui.get_mut(view.app_id()).unwrap(),
                    listener,
                    &mut view.last_pointer_pos,
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

        let result = world
            .query::<&UserInterfaceView>()
            .iter()
            .map(|(_, v)| unsafe { UnsafeRef::upgrade(&scope, v) })
            .collect::<Vec<_>>();
        result
    };

    let dt = universe
        .expect_resource::<AppLifeCycle>()
        .delta_time_seconds();
    let mut context = ProcessContext::new();
    let extras = universe.query_resources::<Q>();
    extras.feed_process_context(&mut context);

    for view in meta {
        if let Some(data) = ui.data.get_mut(unsafe { view.read().app_id() }) {
            data.application.animations_delta_time = dt;
            data.application.process_with_context(&mut context);
            data.application
                .layout(&data.coords_mapping, &mut DefaultLayoutEngine)
                .unwrap_or_default();
            data.interactions.deselect_when_no_button_found =
                unsafe { view.read().deselect_when_no_button_found };
            let _ = data.application.interact(&mut data.interactions);
            data.signals_received = data.application.consume_signals();
        }
    }
}

fn apply_inputs(
    data: &mut ApplicationData,
    listener: &InputStackListener,
    last_pointer_pos: &mut Vec2,
) {
    let pointer_pos = listener.axes_state_or_default::<2>(NAV_POINTER_AXES);
    let pointer_pos = Vec2 {
        x: pointer_pos[0],
        y: pointer_pos[1],
    };
    let pointer_moved = (pointer_pos.x - last_pointer_pos.x).abs() > 1.0e-6
        || (pointer_pos.y - last_pointer_pos.y).abs() > 1.0e-6;
    *last_pointer_pos = pointer_pos;
    let pointer_pos = data.coords_mapping.real_to_virtual_vec2(pointer_pos, false);
    if pointer_moved {
        data.interactions
            .interact(Interaction::PointerMove(pointer_pos));
    }

    let trigger = listener.trigger_state_or_default(NAV_POINTER_ACTION_TRIGGER);
    if trigger.is_pressed() {
        data.interactions.interact(Interaction::PointerDown(
            PointerButton::Trigger,
            pointer_pos,
        ));
    } else if trigger.is_released() {
        data.interactions
            .interact(Interaction::PointerUp(PointerButton::Trigger, pointer_pos));
    }

    let trigger = listener.trigger_state_or_default(NAV_POINTER_CONTEXT_TRIGGER);
    if trigger.is_pressed() {
        data.interactions.interact(Interaction::PointerDown(
            PointerButton::Context,
            pointer_pos,
        ));
    } else if trigger.is_released() {
        data.interactions
            .interact(Interaction::PointerUp(PointerButton::Context, pointer_pos));
    }

    for c in listener.text_state_or_default().chars() {
        if !c.is_control() {
            data.interactions
                .interact(Interaction::Navigate(NavSignal::TextChange(
                    NavTextChange::InsertCharacter(c),
                )));
        } else if c == '\n' || c == '\r' {
            data.interactions
                .interact(Interaction::Navigate(NavSignal::TextChange(
                    NavTextChange::NewLine,
                )));
        }
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_MOVE_CURSOR_LEFT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::MoveCursorLeft,
            )));
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_MOVE_CURSOR_RIGHT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::MoveCursorRight,
            )));
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_MOVE_CURSOR_START_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::MoveCursorStart,
            )));
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_MOVE_CURSOR_END_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::MoveCursorEnd,
            )));
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_DELETE_LEFT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::DeleteLeft,
            )));
    }
    if listener
        .trigger_state_or_default(NAV_TEXT_DELETE_RIGHT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::TextChange(
                NavTextChange::DeleteRight,
            )));
    }

    let trigger = listener.trigger_state_or_default(NAV_ACCEPT_TRIGGER);
    if trigger.is_pressed() {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Accept(true)));
    } else if trigger.is_released() {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Accept(false)));
    }

    let trigger = listener.trigger_state_or_default(NAV_CANCEL_TRIGGER);
    if trigger.is_pressed() {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Cancel(true)));
    } else if trigger.is_released() {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Cancel(false)));
    }

    if listener
        .trigger_state_or_default(NAV_UP_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Up));
    }
    if listener
        .trigger_state_or_default(NAV_DOWN_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Down));
    }
    if listener
        .trigger_state_or_default(NAV_LEFT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Left));
    }
    if listener
        .trigger_state_or_default(NAV_RIGHT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Right));
    }
    if listener
        .trigger_state_or_default(NAV_PREV_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Prev));
    }
    if listener
        .trigger_state_or_default(NAV_NEXT_TRIGGER)
        .is_pressed()
    {
        data.interactions
            .interact(Interaction::Navigate(NavSignal::Next));
    }
}
