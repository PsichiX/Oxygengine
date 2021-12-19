pub mod avatar_combat;
pub mod avatar_movement;
pub mod blink;
pub mod enemy;
pub mod health;
pub mod player;
pub mod weapon;

use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct BatchedSecretsTag;

impl Prefab for BatchedSecretsTag {}
impl PrefabComponent for BatchedSecretsTag {}

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct BatchedAttacksTag;

impl Prefab for BatchedAttacksTag {}
impl PrefabComponent for BatchedAttacksTag {}
