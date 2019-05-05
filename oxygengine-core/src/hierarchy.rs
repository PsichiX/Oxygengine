use specs::{Component, Entity, FlaggedStorage, VecStorage, World};
use specs_hierarchy::Hierarchy;
use std::borrow::Cow;

pub fn hierarchy_find(mut root: Entity, path: &str, world: &World) -> Option<Entity> {
    let hierarchy = world.read_resource::<HierarchyRes>();
    let names = world.read_storage::<Name>();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if let Some(parent) = hierarchy.parent(root) {
                    root = parent;
                } else {
                    return None;
                }
            }
            part => {
                let found = hierarchy.children(root).iter().find_map(|child| {
                    if let Some(name) = names.get(*child) {
                        if name.0 == part {
                            Some(child)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                if let Some(child) = found {
                    root = *child;
                } else {
                    return None;
                }
            }
        }
    }
    Some(root)
}

pub type HierarchyRes = Hierarchy<Parent>;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Parent(pub Entity);

impl Component for Parent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
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

#[derive(Debug, Clone)]
pub struct Name(pub Cow<'static, str>);

impl Component for Name {
    type Storage = VecStorage<Self>;
}
