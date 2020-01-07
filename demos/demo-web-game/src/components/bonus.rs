use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BonusType {
    Ammo(usize),
    Health(usize),
}

#[derive(Debug, Copy, Clone)]
pub struct Bonus(pub BonusType);

impl Component for Bonus {
    type Storage = VecStorage<Self>;
}
