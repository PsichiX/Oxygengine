use crate::{
    prefab::{Prefab, PrefabComponent, PrefabError, PrefabProxy},
    state::StateToken,
};
use serde::{Deserialize, Serialize};
use specs::{
    world::EntitiesRes, Component, Entity, FlaggedStorage, Join, ReadStorage, VecStorage, World,
    WriteStorage,
};
use specs_hierarchy::Hierarchy;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

pub fn entity_find_world(name: &str, world: &World) -> Option<Entity> {
    let entities = world.read_resource::<EntitiesRes>();
    let names = world.read_storage::<Name>();
    entity_find_direct(name, &entities, &names)
}

pub fn entity_find_direct<'s>(
    name: &str,
    entities: &EntitiesRes,
    names: &ReadStorage<'s, Name>,
) -> Option<Entity> {
    (entities, names)
        .join()
        .find_map(|(e, n)| if n.0 == name { Some(e) } else { None })
}

pub fn hierarchy_find_world(root: Entity, path: &str, world: &World) -> Option<Entity> {
    let hierarchy = world.read_resource::<HierarchyRes>();
    let names = world.read_storage::<Name>();
    hierarchy_find_direct(root, path, &hierarchy, &names)
}

pub fn hierarchy_find_direct<'s>(
    mut root: Entity,
    path: &str,
    hierarchy: &HierarchyRes,
    names: &ReadStorage<'s, Name>,
) -> Option<Entity> {
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

pub struct ComponentContainer<'a, C>
where
    C: Component,
{
    entity: Entity,
    storage: WriteStorage<'a, C>,
}

impl<'a, C> ComponentContainer<'a, C>
where
    C: Component,
{
    pub fn new(world: &'a World, entity: Entity) -> Self {
        Self {
            entity,
            storage: world.write_storage::<C>(),
        }
    }

    pub fn get(&self) -> Option<&C> {
        self.storage.get(self.entity)
    }

    pub fn get_mut(&mut self) -> Option<&mut C> {
        self.storage.get_mut(self.entity)
    }

    pub fn unwrap(&self) -> &C {
        self.get().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut C {
        self.get_mut().unwrap()
    }
}

impl<'a, C> Deref for ComponentContainer<'a, C>
where
    C: Component,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        if let Some(c) = self.storage.get(self.entity) {
            c
        } else {
            panic!("Could not fetch component: {}", std::any::type_name::<C>());
        }
    }
}

impl<'a, C> DerefMut for ComponentContainer<'a, C>
where
    C: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(c) = self.storage.get_mut(self.entity) {
            c
        } else {
            panic!(
                "Could not fetch mutable component: {}",
                std::any::type_name::<C>()
            );
        }
    }
}

pub trait ComponentContainerModify<'a, T> {
    fn fetch(world: &'a World, entity: Entity) -> T;
}

impl<'a, C> ComponentContainerModify<'a, ComponentContainer<'a, C>> for C
where
    C: Component,
{
    fn fetch(world: &'a World, entity: Entity) -> ComponentContainer<'a, C> {
        ComponentContainer::<C>::new(world, entity)
    }
}

macro_rules! impl_component_container_modify {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> ComponentContainerModify<'a, ( $( ComponentContainer<'a, $ty> , )* )> for ( $( $ty , )* )
        where $($ty: Component),*
        {
            fn fetch(world: &'a World, entity: Entity) -> ( $( ComponentContainer<'a, $ty> , )* ) {
                ( $( ComponentContainer::<$ty>::new(world, entity), )* )
            }
        }
    };
}

impl_component_container_modify!(A);
impl_component_container_modify!(A, B);
impl_component_container_modify!(A, B, C);
impl_component_container_modify!(A, B, C, D);
impl_component_container_modify!(A, B, C, D, E);
impl_component_container_modify!(A, B, C, D, E, F);
impl_component_container_modify!(A, B, C, D, E, F, G);
impl_component_container_modify!(A, B, C, D, E, F, G, H);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_component_container_modify!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_component_container_modify!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_component_container_modify!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_component_container_modify!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_component_container_modify!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

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
