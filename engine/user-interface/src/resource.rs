use raui_core::{
    application::Application, interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::CoordsMapping, signals::Signal,
};
use std::collections::HashMap;

pub mod input_mappings {
    pub const NAV_POINTER_AXES: &str = "nav-pointer";
    pub const NAV_POINTER_ACTION_TRIGGER: &str = "nav-pointer-action";
    pub const NAV_POINTER_CONTEXT_TRIGGER: &str = "nav-pointer-context";
    pub const NAV_ACCEPT_TRIGGER: &str = "nav-accept";
    pub const NAV_CANCEL_TRIGGER: &str = "nav-cancel";
    pub const NAV_UP_TRIGGER: &str = "nav-up";
    pub const NAV_DOWN_TRIGGER: &str = "nav-down";
    pub const NAV_LEFT_TRIGGER: &str = "nav-left";
    pub const NAV_RIGHT_TRIGGER: &str = "nav-right";
    pub const NAV_PREV_TRIGGER: &str = "nav-prev";
    pub const NAV_NEXT_TRIGGER: &str = "nav-next";
    pub const NAV_TEXT_MOVE_CURSOR_LEFT_TRIGGER: &str = "nav-text-move-cursor-left";
    pub const NAV_TEXT_MOVE_CURSOR_RIGHT_TRIGGER: &str = "nav-text-move-cursor-right";
    pub const NAV_TEXT_MOVE_CURSOR_START_TRIGGER: &str = "nav-text-move-cursor-start";
    pub const NAV_TEXT_MOVE_CURSOR_END_TRIGGER: &str = "nav-text-move-cursor-end";
    pub const NAV_TEXT_DELETE_LEFT_TRIGGER: &str = "nav-text-delete-left";
    pub const NAV_TEXT_DELETE_RIGHT_TRIGGER: &str = "nav-text-delete-right";
}

#[derive(Default)]
pub struct ApplicationData {
    pub application: Application,
    pub interactions: DefaultInteractionsEngine,
    pub coords_mapping: CoordsMapping,
    pub(crate) signals_received: Vec<Signal>,
}

impl ApplicationData {
    pub fn signals_received(&self) -> &[Signal] {
        &self.signals_received
    }
}

#[derive(Default)]
pub struct UserInterface {
    pub(crate) data: HashMap<String, ApplicationData>,
    pub(crate) setup_application: Option<fn(&mut Application)>,
}

impl UserInterface {
    pub fn new(setup_application: fn(&mut Application)) -> Self {
        Self {
            data: Default::default(),
            setup_application: Some(setup_application),
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ApplicationData)> {
        self.data.iter().map(|(n, d)| (n.as_str(), d))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut ApplicationData)> {
        self.data.iter_mut().map(|(n, d)| (n.as_str(), d))
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

    #[inline]
    pub fn signals_received(&self, app_id: &str) -> Option<&[Signal]> {
        self.data.get(app_id).map(|item| item.signals_received())
    }

    #[inline]
    pub fn all_signals_received(&self) -> impl Iterator<Item = (&str, &Signal)> {
        self.data.iter().flat_map(|(id, item)| {
            item.signals_received()
                .iter()
                .map(move |signal| (id.as_str(), signal))
        })
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
