use oxygengine::prelude::*;

#[derive(Default)]
pub struct TurnManager {
    entities: Vec<Entity>,
    active: Option<Entity>,
}

impl TurnManager {
    pub fn register(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn unregister(&mut self, entity: Entity) {
        if let Some(index) = self.entities.iter().position(|e| *e == entity) {
            self.entities.remove(index);
        }
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn selected(&self) -> Option<Entity> {
        self.active
    }

    pub fn select(&mut self, entity: Entity) {
        if self.entities.contains(&entity) {
            self.active = Some(entity);
        }
    }

    pub fn deselect(&mut self) {
        self.active = None;
    }

    pub fn select_nth(&mut self, index: usize) {
        if let Some(entity) = self.entities.get(index) {
            self.active = Some(*entity);
        }
    }

    pub fn select_next(&mut self) {
        if self.entities.is_empty() {
            if let Some(entity) = self.active {
                if let Some(index) = self.entities.iter().position(|e| *e == entity) {
                    let e = self.entities[(index + 1) % self.entities.len()];
                    self.active = Some(e);
                    return;
                }
            } else {
                self.active = Some(self.entities[0]);
                return;
            }
        }
        self.active = None;
    }
}
