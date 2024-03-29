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
use std::collections::HashMap;

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
    roots: Vec<Entity>,
    child_parent_relations: HashMap<Entity, Entity>,
    parent_children_relations: HashMap<Entity, Vec<Entity>>,
    entity_names_map: HashMap<Entity, String>,
    name_entities_map: HashMap<String, Entity>,
}

impl Hierarchy {
    pub fn roots(&self) -> impl Iterator<Item = Entity> + '_ {
        self.roots.iter().copied()
    }

    pub fn parents(&self) -> impl Iterator<Item = Entity> + '_ {
        self.parent_children_relations.keys().copied()
    }

    pub fn childs(&self) -> impl Iterator<Item = Entity> + '_ {
        self.child_parent_relations.keys().copied()
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

    pub fn iter(&self) -> HierarchyIter {
        HierarchyIter {
            hierarchy: self,
            iter_stack: vec![Box::new(self.roots.iter().copied())],
            parent_stack: vec![],
        }
    }
}

pub struct HierarchyIter<'a> {
    hierarchy: &'a Hierarchy,
    iter_stack: Vec<Box<dyn Iterator<Item = Entity> + 'a>>,
    parent_stack: Vec<Entity>,
}

impl<'a> Iterator for HierarchyIter<'a> {
    type Item = (Entity, Option<Entity>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(mut iter) = self.iter_stack.pop() {
                if let Some(entity) = iter.next() {
                    self.iter_stack.push(iter);
                    if let Some(children) = self.hierarchy.children(entity) {
                        self.iter_stack.push(Box::new(children));
                    }
                    return Some((entity, self.parent_stack.last().copied()));
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }
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

        hierarchy.roots = Vec::with_capacity(world.len() as usize);
        hierarchy.child_parent_relations = HashMap::with_capacity(world.len() as usize);
        hierarchy.parent_children_relations = HashMap::with_capacity(world.len() as usize / 10);
        hierarchy.entity_names_map = HashMap::with_capacity(world.len() as usize / 10);
        hierarchy.name_entities_map = HashMap::with_capacity(world.len() as usize / 10);
    } else {
        hierarchy.roots.clear();
        hierarchy.child_parent_relations.clear();
        hierarchy.parent_children_relations.clear();
        hierarchy.entity_names_map.clear();
        hierarchy.name_entities_map.clear();
    }

    for (entity, _) in world.query::<()>().without::<&Parent>().iter() {
        hierarchy.roots.push(entity);
    }

    for (child, (parent, name)) in world.query::<(Option<&Parent>, Option<&Name>)>().iter() {
        if let Some(parent) = parent {
            hierarchy.child_parent_relations.insert(child, parent.0);
            let list: &mut Vec<Entity> = hierarchy
                .parent_children_relations
                .entry(parent.0)
                .or_default();
            list.push(child);
        }
        if let Some(name) = name {
            let name: String = name.0.clone().into();
            hierarchy.entity_names_map.insert(child, name.to_owned());
            hierarchy.name_entities_map.insert(name, child);
        }
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
