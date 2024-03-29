use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Weapon(pub usize);

impl Prefab for Weapon {}
impl PrefabComponent for Weapon {}
