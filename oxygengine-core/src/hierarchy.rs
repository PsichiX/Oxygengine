use crate::{
    prefab::{Prefab, PrefabComponent, PrefabError, PrefabProxy},
    state::StateToken,
};
use serde::{Deserialize, Serialize};
use specs::{Component, Entity, FlaggedStorage, VecStorage, World};
use specs_hierarchy::Hierarchy;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

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

#[derive(Default)]
pub struct HierarchyChangeRes {
    pub(crate) entities: HashSet<Entity>,
    pub(crate) added: Vec<Entity>,
    pub(crate) removed: Vec<Entity>,
}

impl HierarchyChangeRes {
    #[inline]
    pub fn added(&self) -> &[Entity] {
        &self.added
    }

    #[inline]
    pub fn removed(&self) -> &[Entity] {
        &self.removed
    }
}

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

impl PrefabProxy<ParentPrefabProxy> for Parent {
    fn from_proxy_with_extras(
        proxy: ParentPrefabProxy,
        named_entities: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        if let Some(entity) = named_entities.get(&proxy.0) {
            Ok(Self(*entity))
        } else {
            Err(PrefabError::Custom(format!(
                "Could not find entity named: {}",
                proxy.0
            )))
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ParentPrefabProxy(pub String);

impl Prefab for ParentPrefabProxy {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tag(pub Cow<'static, str>);

impl Component for Tag {
    type Storage = VecStorage<Self>;
}

impl Prefab for Tag {}
impl PrefabComponent for Tag {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Name(pub Cow<'static, str>);

impl Component for Name {
    type Storage = VecStorage<Self>;
}

impl Prefab for Name {}
impl PrefabComponent for Name {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NonPersistent(pub StateToken);

impl Component for NonPersistent {
    type Storage = VecStorage<Self>;
}

impl PrefabProxy<NonPersistentPrefabProxy> for NonPersistent {
    fn from_proxy_with_extras(
        _: NonPersistentPrefabProxy,
        _: &HashMap<String, Entity>,
        state_token: StateToken,
    ) -> Result<Self, PrefabError> {
        Ok(NonPersistent(state_token))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NonPersistentPrefabProxy;

impl Prefab for NonPersistentPrefabProxy {}
