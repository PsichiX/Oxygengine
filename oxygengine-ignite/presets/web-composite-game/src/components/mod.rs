use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

pub mod speed;

// component that tags entity as moved with keyboard.
#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct KeyboardMovementTag;

impl Component for KeyboardMovementTag {
    // tag components are empty so they use `NullStorage`.
    type Storage = NullStorage<Self>;
}

impl Prefab for KeyboardMovementTag {}
impl PrefabComponent for KeyboardMovementTag {}
