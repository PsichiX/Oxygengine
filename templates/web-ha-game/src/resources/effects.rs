use oxygengine::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Effects {
    attacks: HashMap<Location, Scalar>,
    secrets: HashSet<Location>,
}

impl Effects {
    pub fn with_capacity(attacks_capacity: usize, secrets_capacity: usize) -> Self {
        Self {
            attacks: HashMap::with_capacity(attacks_capacity),
            secrets: HashSet::with_capacity(secrets_capacity),
        }
    }

    pub fn attacks(&self) -> impl Iterator<Item = Location> + '_ {
        self.attacks.keys().copied()
    }

    pub fn secrets(&self) -> impl Iterator<Item = Location> + '_ {
        self.secrets.iter().copied()
    }

    pub fn attack(&mut self, location: Location, duration: Scalar) {
        self.attacks.insert(location, duration);
    }

    pub fn secret(&mut self, location: Location) {
        self.secrets.insert(location);
    }

    pub fn maintain(&mut self, delta_time: Scalar) {
        for timer in self.attacks.values_mut() {
            *timer -= delta_time;
        }
        self.attacks.retain(|_, timer| *timer > 0.0);
        self.secrets.clear();
    }
}
