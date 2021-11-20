use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinBotOwner {
    LocalPlayer(usize),
    RemotePlayer(usize),
    Bot,
}

impl Default for SpinBotOwner {
    fn default() -> Self {
        Self::Bot
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpinBotStats {
    pub radius: Scalar,
    pub mass: Scalar,
    #[serde(default)]
    pub friction_coefficient: Scalar,
    #[serde(default)]
    pub restitution_coefficient: Scalar,
    #[serde(default)]
    pub linear_damping: Scalar,
    #[serde(default)]
    pub angular_damping: Scalar,
}

impl SpinBotStats {
    pub fn combine<'a>(parts: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut result = Self::default();
        let mut count = 0;
        for part in parts {
            count += 1;
            result.radius = result.radius.max(part.radius);
            result.mass += part.mass;
            result.friction_coefficient += part.friction_coefficient.max(0.0);
            result.restitution_coefficient += part.restitution_coefficient.max(0.0);
            result.linear_damping += part.linear_damping;
            result.angular_damping += part.angular_damping;
        }
        if count > 0 {
            result.restitution_coefficient = (result.restitution_coefficient / count as Scalar)
                .max(0.0)
                .min(1.0);
        }
        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinBotAbility {
    None,
    /// [Stats buff] Higher attack and defense.
    Berserk,
    /// [Stats buff] Higher defense and stamina.
    Immortality,
    /// [Stats buff] Higher stamina and attack.
    Marathon,
    /// [Defensive buff] Makes opponents unable to control their spinbots.
    RadioJamming,
    /// [Defensive buff] Instant opponents power mode deactivation.
    EMP,
    /// [Defensive buff] Barrier that deflects incoming opponents.
    Barrier,
    /// [Offensive buff] Makes opponents attracted to user.
    Magnet,
    /// [Offensive buff] Makes user move to the center of the arena.
    Gravity,
    /// [Offensive buff] Makes nearby opponents lose more momentum.
    Wobble,
}

impl Default for SpinBotAbility {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SpinBotPower {
    /// Current power accumulation phase.
    Charge(Scalar),
    /// Time left to stop power mode.
    Active(Scalar),
}

impl Default for SpinBotPower {
    fn default() -> Self {
        Self::Charge(0.0)
    }
}
