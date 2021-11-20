use crate::utils::spinbot::*;
use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpinBot {
    pub owner: SpinBotOwner,
    pub ability: SpinBotAbility,
    pub power: SpinBotPower,
}

impl Prefab for SpinBot {}
impl PrefabComponent for SpinBot {}

impl SpinBot {
    pub fn active_control(&self) -> bool {
        matches!(self.power, SpinBotPower::Active(_))
    }

    pub fn active_ability(&self) -> Option<SpinBotAbility> {
        match self.power {
            SpinBotPower::Charge(_) => None,
            SpinBotPower::Active(_) => Some(self.ability),
        }
    }
}
