use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub enum AvatarCombatMode {
    /// (points)
    Attack(usize),
    /// (seconds left)
    Cooldown(Scalar),
}

impl Default for AvatarCombatMode {
    fn default() -> Self {
        Self::Cooldown(0.0)
    }
}

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AvatarCombat {
    #[serde(default = "AvatarCombat::default_cooldown")]
    pub cooldown: Scalar,
    #[serde(skip)]
    #[ignite(ignore)]
    mode: AvatarCombatMode,
}

impl Default for AvatarCombat {
    fn default() -> Self {
        Self {
            cooldown: Self::default_cooldown(),
            mode: Default::default(),
        }
    }
}

impl AvatarCombat {
    fn default_cooldown() -> Scalar {
        0.5
    }

    pub fn is_ready(&self) -> bool {
        match self.mode {
            AvatarCombatMode::Cooldown(v) => v < 1.0e-6,
            _ => false,
        }
    }

    pub fn try_attack(&mut self, points: usize) -> bool {
        if self.is_ready() {
            self.mode = AvatarCombatMode::Attack(points);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn process(&mut self, delta_time: Scalar) -> Option<usize> {
        match self.mode {
            AvatarCombatMode::Attack(points) => {
                self.mode = AvatarCombatMode::Cooldown(self.cooldown);
                Some(points)
            }
            AvatarCombatMode::Cooldown(v) => {
                self.mode = AvatarCombatMode::Cooldown((v - delta_time).max(0.0));
                None
            }
        }
    }
}

impl Prefab for AvatarCombat {}

impl PrefabComponent for AvatarCombat {}
