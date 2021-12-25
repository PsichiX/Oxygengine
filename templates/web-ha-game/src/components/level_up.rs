use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct LevelUp(pub usize);

impl Prefab for LevelUp {}
impl PrefabComponent for LevelUp {}
