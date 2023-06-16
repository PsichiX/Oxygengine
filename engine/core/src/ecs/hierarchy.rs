use crate::{
    ecs::{
        commands::{DespawnEntity, UniverseCommands},
        components::Name,
        life_cycle::EntityChanges,
        Comp, Entity, Universe, WorldRef,
    },
    prefab::{Prefab, PrefabError, PrefabProxy},
    state::StateToken,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Parent(pub Entity);

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

#[derive(Debug, Default)]
pub struct Hierarchy {
    child_parent_relations: HashMap<Entity, Entity>,
    parent_children_relations: HashMap<Entity, HashSet<Entity>>,
    entity_names_map: HashMap<Entity, String>,
    name_entities_map: HashMap<String, Entity>,
}

impl Hierarchy {
    pub fn roots(&self) -> impl Iterator<Item = Entity> + '_ {
        self.parent_children_relations.keys().copied()
    }

    pub fn parent(&self, child: Entity) -> Option<Entity> {
        self.child_parent_relations.get(&child).copied()
    }

    pub fn children(&self, parent: Entity) -> Option<impl Iterator<Item = Entity> + '_> {
        self.parent_children_relations
            .get(&parent)
            .map(|list| list.iter().copied())
    }

    pub fn entity_by_name(&self, name: &str) -> Option<Entity> {
        self.name_entities_map.get(name).copied()
    }

    pub fn name_by_entity(&self, entity: Entity) -> Option<&str> {
        self.entity_names_map.get(&entity).map(|name| name.as_str())
    }

    pub fn find(&self, root: Option<Entity>, mut path: &str) -> Option<Entity> {
        let mut root = match root {
            Some(root) => root,
            None => {
                let part = match path.find('/') {
                    Some(found) => {
                        let part = &path[..found];
                        path = &path[(found + 1)..];
                        part
                    }
                    None => {
                        let part = path;
                        path = "";
                        part
                    }
                };
                match self.entity_by_name(part) {
                    Some(root) => root,
                    None => return None,
                }
            }
        };
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => match self.parent(root) {
                    Some(parent) => root = parent,
                    None => return None,
                },
                part => match self.children(root) {
                    Some(mut iter) => match iter.find(|child| {
                        self.name_by_entity(*child)
                            .map(|name| name == part)
                            .unwrap_or_default()
                    }) {
                        Some(child) => root = child,
                        None => return None,
                    },
                    None => return None,
                },
            }
        }
        Some(root)
    }
}

pub type HierarchySystemResources<'a> = (
    WorldRef,
    &'a mut UniverseCommands,
    &'a EntityChanges,
    &'a mut Hierarchy,
    Comp<&'a Parent>,
    Comp<&'a Name>,
);

pub fn hierarchy_system(universe: &mut Universe) {
    let (world, mut commands, changes, mut hierarchy, ..) =
        universe.query_resources::<HierarchySystemResources>();

    if changes.has_changed() {
        despawn(&mut commands, &changes, &hierarchy);

        hierarchy.child_parent_relations = HashMap::with_capacity(world.len() as usize);
        hierarchy.parent_children_relations = HashMap::with_capacity(world.len() as usize / 10);
        hierarchy.entity_names_map = HashMap::with_capacity(world.len() as usize / 10);
        hierarchy.name_entities_map = HashMap::with_capacity(world.len() as usize / 10);
    } else {
        hierarchy.child_parent_relations.clear();
        hierarchy.parent_children_relations.clear();
        hierarchy.entity_names_map.clear();
        hierarchy.name_entities_map.clear();
    }

    for (child, (parent, name)) in world.query::<(Option<&Parent>, Option<&Name>)>().iter() {
        if let Some(parent) = parent {
            hierarchy.child_parent_relations.insert(child, parent.0);
            let list = hierarchy
                .parent_children_relations
                .entry(parent.0)
                .or_default();
            list.insert(child);
        }
        if let Some(name) = name {
            let name: String = name.0.to_owned().into();
            hierarchy.entity_names_map.insert(child, name.to_owned());
            hierarchy.name_entities_map.insert(name, child);
        }
    }

    for (child, parent) in world.query::<&Parent>().iter() {
        hierarchy.child_parent_relations.insert(child, parent.0);
        let list = hierarchy
            .parent_children_relations
            .entry(parent.0)
            .or_default();
        list.insert(child);
    }

    for (child, name) in world.query::<&Name>().iter() {
        hierarchy
            .entity_names_map
            .insert(child, name.0.as_ref().to_owned());
        hierarchy
            .name_entities_map
            .insert(name.0.as_ref().to_owned(), child);
    }
}

fn despawn(commands: &mut UniverseCommands, changes: &EntityChanges, hierarchy: &Hierarchy) {
    for entity in changes.despawned() {
        despawn_children(commands, entity, hierarchy);
    }
}

fn despawn_children(commands: &mut UniverseCommands, parent: Entity, hierarchy: &Hierarchy) {
    if let Some(iter) = hierarchy.children(parent) {
        for entity in iter {
            commands.schedule(DespawnEntity(entity));
            despawn_children(commands, entity, hierarchy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::Component;

    #[test]
    fn test_send_sync() {
        fn foo<T>()
        where
            T: Component + Send + Sync,
        {
            println!("{} is Component", std::any::type_name::<T>());
        }

        foo::<Parent>();
    }
}
