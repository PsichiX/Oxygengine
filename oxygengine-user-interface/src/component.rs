use core::{
    ecs::{Component, FlaggedStorage, VecStorage},
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use raui_core::PrefabValue;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct UserInterfaceView {
    #[serde(default)]
    app_id: String,
    #[serde(default)]
    root: PrefabValue,
    #[serde(default)]
    pub input_order: usize,
    #[serde(default)]
    pub capture_input: bool,
    #[serde(default)]
    pub deselect_when_no_button_found: bool,
    #[serde(skip)]
    #[serde(default = "UserInterfaceView::default_dirty")]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl Default for UserInterfaceView {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl UserInterfaceView {
    fn default_dirty() -> bool {
        true
    }

    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            root: Default::default(),
            input_order: 0,
            capture_input: false,
            deselect_when_no_button_found: false,
            dirty: true,
        }
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn root(&self) -> &PrefabValue {
        &self.root
    }

    pub fn set_root(&mut self, root: PrefabValue) {
        self.root = root;
        self.dirty = true;
    }
}

impl Component for UserInterfaceView {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Prefab for UserInterfaceView {}
impl PrefabComponent for UserInterfaceView {}
