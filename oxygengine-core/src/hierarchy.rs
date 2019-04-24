use specs::{Component, DenseVecStorage, Entity, FlaggedStorage};
use specs_hierarchy::Hierarchy;

pub type HierarchyRes = Hierarchy<Parent>;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Parent(pub Entity);

impl Component for Parent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl specs_hierarchy::Parent for Parent {
    fn parent_entity(&self) -> Entity {
        self.0
    }
}
