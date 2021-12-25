use oxygengine::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Default, Copy, Clone)]
pub struct SecretEffect {
    pub health: usize,
    pub weapons: usize,
}

impl SecretEffect {
    pub fn is_valid(&self) -> bool {
        self.health > 0 || self.weapons > 0
    }
}

#[derive(Debug, Default)]
pub struct Effects {
    attacks: HashMap<Location, Scalar>,
    secrets: HashMap<Location, SecretEffect>,
}

impl Effects {
    pub fn with_capacity(attacks_capacity: usize, secrets_capacity: usize) -> Self {
        Self {
            attacks: HashMap::with_capacity(attacks_capacity),
            secrets: HashMap::with_capacity(secrets_capacity),
        }
    }

    pub fn attacks(&self) -> impl Iterator<Item = Location> + '_ {
        self.attacks.keys().copied()
    }

    pub fn secrets(&self) -> impl Iterator<Item = (Location, SecretEffect)> + '_ {
        self.secrets
            .iter()
            .map(|(location, secret)| (*location, *secret))
    }

    pub fn attack(&mut self, location: Location, duration: Scalar) {
        self.attacks.insert(location, duration);
    }

    pub fn clear_secrets(&mut self) {
        self.secrets.clear();
    }

    pub fn add_secret(&mut self, location: Location, secret: SecretEffect) {
        self.secrets.insert(location, secret);
    }

    pub fn remove_secret(&mut self, location: Location) -> Option<SecretEffect> {
        self.secrets.remove(&location)
    }

    pub fn maintain(&mut self, delta_time: Scalar) {
        for timer in self.attacks.values_mut() {
            *timer -= delta_time;
        }
        self.attacks.retain(|_, timer| *timer > 0.0);
    }
}
