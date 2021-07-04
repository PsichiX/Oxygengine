use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

pub mod speed;

// component that tags entity as moved with keyboard.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct KeyboardMovementTag;

impl Prefab for KeyboardMovementTag {}
impl PrefabComponent for KeyboardMovementTag {}
