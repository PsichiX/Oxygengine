use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player;

impl Prefab for Player {}
impl PrefabComponent for Player {}
