use specs::{Component, DenseVecStorage, Entity, FlaggedStorage, VecStorage};
use specs_hierarchy::Hierarchy;
use std::borrow::Cow;

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

#[derive(Debug, Clone)]
pub struct Tag(pub Cow<'static, str>);

impl Component for Tag {
    type Storage = VecStorage<Self>;
}
