use oxygengine_core::prelude::*;
use oxygengine_overworld::resources::board::BoardLocation;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct HaBoardChunkSync(pub BoardLocation);

impl Prefab for HaBoardChunkSync {}

impl PrefabComponent for HaBoardChunkSync {}
