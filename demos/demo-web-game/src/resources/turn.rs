use oxygengine::prelude::*;

const WAITING_TIME: f64 = 3.0;
const PLAYING_TIME: f64 = 9.0;

#[derive(Debug, Copy, Clone)]
pub enum Timer {
    None,
    Waiting(f64),
    Playing(f64),
}

impl Default for Timer {
    fn default() -> Self {
        Self::None
    }
}

impl Timer {
    pub fn is_playing(self) -> bool {
        if let Self::Playing(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct TurnManager {
    entities: Vec<Entity>,
    active: Option<Entity>,
    timer: Timer,
}

impl TurnManager {
    pub fn register(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn unregister(&mut self, entity: Entity) {
        if let Some(index) = self.entities.iter().position(|e| *e == entity) {
            self.entities.remove(index);
            if let Some(active) = self.active {
                if active == entity {
                    self.select_next();
                }
            }
        }
    }

    pub fn timer(&self) -> Timer {
        self.timer
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.active = None;
        self.timer = Timer::None;
    }

    pub fn selected(&self) -> Option<Entity> {
        self.active
    }

    pub fn selected_playing(&self) -> Option<Entity> {
        if self.timer.is_playing() {
            self.active
        } else {
            None
        }
    }

    pub fn select_nth(&mut self, index: usize) {
        if let Some(entity) = self.entities.get(index) {
            self.timer = Timer::Waiting(WAITING_TIME);
            self.active = Some(*entity);
        }
    }

    pub fn select_next(&mut self) {
        if !self.entities.is_empty() {
            if let Some(entity) = self.active {
                if let Some(index) = self.entities.iter().position(|e| *e == entity) {
                    let e = self.entities[(index + 1) % self.entities.len()];
                    self.timer = Timer::Waiting(WAITING_TIME);
                    self.active = Some(e);
                    return;
                }
            } else {
                self.timer = Timer::Waiting(WAITING_TIME);
                self.active = Some(self.entities[0]);
                return;
            }
        }
        self.active = None;
        self.timer = Timer::None;
    }

    pub fn process(&mut self, delta_time: f64) {
        match self.timer {
            Timer::Waiting(mut t) => {
                t -= delta_time;
                if t <= 0.0 {
                    self.timer = Timer::Playing(PLAYING_TIME);
                } else {
                    self.timer = Timer::Waiting(t);
                }
            }
            Timer::Playing(mut t) => {
                t -= delta_time;
                if t <= 0.0 {
                    self.select_next();
                } else {
                    self.timer = Timer::Playing(t);
                }
            }
            _ => {}
        }
    }
}
