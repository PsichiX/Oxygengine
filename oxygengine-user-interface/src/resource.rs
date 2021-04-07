use raui_core::{
    application::Application, interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::CoordsMapping,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct ApplicationData {
    pub application: Application,
    pub interactions: DefaultInteractionsEngine,
    pub coords_mapping: CoordsMapping,
}

#[derive(Default)]
pub struct UserInterfaceRes {
    pub(crate) data: HashMap<String, ApplicationData>,
    pub(crate) setup: Option<fn(&mut Application)>,
    pub(crate) last_frame_captured: bool,
    pub pointer_axis_x: String,
    pub pointer_axis_y: String,
    pub pointer_action_trigger: String,
    pub pointer_context_trigger: String,
    pub navigate_accept: String,
    pub navigate_cancel: String,
    pub navigate_up: String,
    pub navigate_down: String,
    pub navigate_left: String,
    pub navigate_right: String,
    pub navigate_prev: String,
    pub navigate_next: String,
    pub text_move_cursor_left: String,
    pub text_move_cursor_right: String,
    pub text_move_cursor_start: String,
    pub text_move_cursor_end: String,
    pub text_delete_left: String,
    pub text_delete_right: String,
}

impl UserInterfaceRes {
    pub fn new(f: fn(&mut Application)) -> Self {
        Self {
            data: Default::default(),
            setup: Some(f),
            last_frame_captured: false,
            pointer_axis_x: Default::default(),
            pointer_axis_y: Default::default(),
            pointer_action_trigger: Default::default(),
            pointer_context_trigger: Default::default(),
            navigate_accept: Default::default(),
            navigate_cancel: Default::default(),
            navigate_up: Default::default(),
            navigate_down: Default::default(),
            navigate_left: Default::default(),
            navigate_right: Default::default(),
            navigate_prev: Default::default(),
            navigate_next: Default::default(),
            text_move_cursor_left: Default::default(),
            text_move_cursor_right: Default::default(),
            text_move_cursor_start: Default::default(),
            text_move_cursor_end: Default::default(),
            text_delete_left: Default::default(),
            text_delete_right: Default::default(),
        }
    }

    pub fn last_frame_captured(&self) -> bool {
        self.last_frame_captured
    }

    pub fn with_pointer_axis(mut self, x: &str, y: &str) -> Self {
        self.pointer_axis_x = x.to_owned();
        self.pointer_axis_y = y.to_owned();
        self
    }

    pub fn with_pointer_trigger(mut self, action: &str, context: &str) -> Self {
        self.pointer_action_trigger = action.to_owned();
        self.pointer_context_trigger = context.to_owned();
        self
    }

    pub fn with_navigation_actions(mut self, accept: &str, cancel: &str) -> Self {
        self.navigate_accept = accept.to_owned();
        self.navigate_cancel = cancel.to_owned();
        self
    }

    pub fn with_navigation_directions(
        mut self,
        up: &str,
        down: &str,
        left: &str,
        right: &str,
    ) -> Self {
        self.navigate_up = up.to_owned();
        self.navigate_down = down.to_owned();
        self.navigate_left = left.to_owned();
        self.navigate_right = right.to_owned();
        self
    }

    pub fn with_navigation_tabs(mut self, prev: &str, next: &str) -> Self {
        self.navigate_prev = prev.to_owned();
        self.navigate_next = next.to_owned();
        self
    }

    pub fn with_text_move_cursor(
        mut self,
        left: &str,
        right: &str,
        start: &str,
        end: &str,
    ) -> Self {
        self.text_move_cursor_left = left.to_owned();
        self.text_move_cursor_right = right.to_owned();
        self.text_move_cursor_start = start.to_owned();
        self.text_move_cursor_end = end.to_owned();
        self
    }

    pub fn with_text_delete(mut self, left: &str, right: &str) -> Self {
        self.text_delete_left = left.to_owned();
        self.text_delete_right = right.to_owned();
        self
    }

    #[inline]
    pub fn get(&self, app_id: &str) -> Option<&ApplicationData> {
        self.data.get(app_id)
    }

    #[inline]
    pub fn get_mut(&mut self, app_id: &str) -> Option<&mut ApplicationData> {
        self.data.get_mut(app_id)
    }

    #[inline]
    pub fn application(&self, app_id: &str) -> Option<&Application> {
        self.data.get(app_id).map(|item| &item.application)
    }

    #[inline]
    pub fn application_mut(&mut self, app_id: &str) -> Option<&mut Application> {
        self.data.get_mut(app_id).map(|item| &mut item.application)
    }

    #[inline]
    pub fn interactions(&self, app_id: &str) -> Option<&DefaultInteractionsEngine> {
        self.data.get(app_id).map(|item| &item.interactions)
    }

    #[inline]
    pub fn interactions_mut(&mut self, app_id: &str) -> Option<&mut DefaultInteractionsEngine> {
        self.data.get_mut(app_id).map(|item| &mut item.interactions)
    }

    #[inline]
    pub fn coords_mapping(&self, app_id: &str) -> Option<&CoordsMapping> {
        self.data.get(app_id).map(|item| &item.coords_mapping)
    }

    #[inline]
    pub fn coords_mapping_mut(&mut self, app_id: &str) -> Option<&mut CoordsMapping> {
        self.data
            .get_mut(app_id)
            .map(|item| &mut item.coords_mapping)
    }

    pub fn has_layout_widget(&self, app_id: &str, id: &str) -> bool {
        if let Some(item) = self.data.get(app_id) {
            item.application
                .layout_data()
                .items
                .keys()
                .any(|k| k.as_ref() == id)
        } else {
            false
        }
    }
}
