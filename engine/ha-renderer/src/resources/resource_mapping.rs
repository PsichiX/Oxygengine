use core::{assets::asset::AssetId, id::ID};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ResourceMappingEntry<R, V> {
    Resource(ID<R>),
    VirtualResource(ID<V>, ID<R>),
}

impl<R, V> ResourceMappingEntry<R, V> {
    pub fn resource(&self) -> Option<ID<R>> {
        match self {
            Self::Resource(id) => Some(*id),
            _ => None,
        }
    }

    pub fn virtual_resource(&self) -> Option<(ID<V>, ID<R>)> {
        match self {
            Self::VirtualResource(vid, rid) => Some((*vid, *rid)),
            _ => None,
        }
    }
}

impl<R, V> Clone for ResourceMappingEntry<R, V> {
    fn clone(&self) -> Self {
        match self {
            Self::Resource(r) => Self::Resource(*r),
            Self::VirtualResource(v, r) => Self::VirtualResource(*v, *r),
        }
    }
}

type ResourceMappingValue<R, V> = (Option<AssetId>, ResourceMappingEntry<R, V>, bool);

#[derive(Debug)]
pub struct ResourceMapping<R, V = ()> {
    map: HashMap<AssetId, String>,
    table: HashMap<String, ResourceMappingValue<R, V>>,
    removed: Vec<String>,
}

impl<R, V> Default for ResourceMapping<R, V> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            table: Default::default(),
            removed: Default::default(),
        }
    }
}

impl<R, V> ResourceMapping<R, V> {
    pub fn has_asset(&self, asset_id: AssetId) -> bool {
        self.map.contains_key(&asset_id)
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn maintain(&mut self) {
        for (_, _, added) in self.table.values_mut() {
            *added = false;
        }
        self.removed.clear();
    }

    pub fn map_asset_entry(
        &mut self,
        name: impl ToString,
        asset_id: AssetId,
        entry: ResourceMappingEntry<R, V>,
    ) {
        self.map.insert(asset_id, name.to_string());
        self.table
            .insert(name.to_string(), (Some(asset_id), entry, true));
    }

    pub fn map_asset_resource(
        &mut self,
        name: impl ToString,
        asset_id: AssetId,
        resource_id: ID<R>,
    ) {
        self.map_asset_entry(name, asset_id, ResourceMappingEntry::Resource(resource_id));
    }

    pub fn map_asset_virtual_resource(
        &mut self,
        name: impl ToString,
        asset_id: AssetId,
        virtual_resource_id: ID<V>,
        resource_id: ID<R>,
    ) {
        self.map_asset_entry(
            name,
            asset_id,
            ResourceMappingEntry::VirtualResource(virtual_resource_id, resource_id),
        );
    }

    pub fn map_entry(&mut self, name: impl ToString, entry: ResourceMappingEntry<R, V>) {
        self.table.insert(name.to_string(), (None, entry, true));
    }

    pub fn map_resource(&mut self, name: impl ToString, resource_id: ID<R>) {
        self.map_entry(name, ResourceMappingEntry::Resource(resource_id));
    }

    pub fn map_virtual_resource(
        &mut self,
        name: impl ToString,
        virtual_resource_id: ID<V>,
        resource_id: ID<R>,
    ) {
        self.map_entry(
            name,
            ResourceMappingEntry::VirtualResource(virtual_resource_id, resource_id),
        );
    }

    pub fn unmap_asset(&mut self, asset_id: AssetId) -> Option<ResourceMappingEntry<R, V>> {
        if let Some(name) = self.map.remove(&asset_id) {
            let result = self.table.remove(&name).map(|(_, entry, _)| entry);
            self.removed.push(name);
            return result;
        }
        None
    }

    pub fn unmap_asset_resource(&mut self, asset_id: AssetId) -> Option<ID<R>> {
        self.unmap_asset(asset_id)?.resource()
    }

    pub fn unmap_asset_virtual_resource(&mut self, asset_id: AssetId) -> Option<(ID<V>, ID<R>)> {
        self.unmap_asset(asset_id)?.virtual_resource()
    }

    pub fn unmap_name(&mut self, name: &str) -> Option<ResourceMappingEntry<R, V>> {
        self.removed.push(name.to_owned());
        self.table.remove(name).map(|(_, entry, _)| entry)
    }

    pub fn unmap_name_resource(&mut self, name: &str) -> Option<ID<R>> {
        self.unmap_name(name)?.resource()
    }

    pub fn unmap_name_virtual_resource(&mut self, name: &str) -> Option<(ID<V>, ID<R>)> {
        self.unmap_name(name)?.virtual_resource()
    }

    pub fn entry_by_name(&self, name: &str) -> Option<&ResourceMappingEntry<R, V>> {
        self.table.get(name).map(|(_, entry, _)| entry)
    }

    pub fn resource_by_name(&self, name: &str) -> Option<ID<R>> {
        self.entry_by_name(name)?.resource()
    }

    pub fn virtual_resource_by_name(&self, name: &str) -> Option<(ID<V>, ID<R>)> {
        self.entry_by_name(name)?.virtual_resource()
    }

    pub fn entry_by_asset(&self, asset_id: AssetId) -> Option<&ResourceMappingEntry<R, V>> {
        if let Some(name) = self.map.get(&asset_id) {
            return self.entry_by_name(name);
        }
        None
    }

    pub fn resource_by_asset(&self, asset_id: AssetId) -> Option<ID<R>> {
        self.entry_by_asset(asset_id)?.resource()
    }

    pub fn virtual_resource_by_asset(&self, asset_id: AssetId) -> Option<(ID<V>, ID<R>)> {
        self.entry_by_asset(asset_id)?.virtual_resource()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &ResourceMappingEntry<R, V>)> {
        self.table
            .iter()
            .map(|(name, (_, entry, _))| (name.as_str(), entry))
    }

    pub fn resources(&self) -> impl Iterator<Item = (&str, ID<R>)> {
        self.table
            .iter()
            .filter_map(|(name, (_, entry, _))| Some((name.as_str(), entry.resource()?)))
    }

    pub fn virtual_resources(&self) -> impl Iterator<Item = (&str, ID<V>, ID<R>)> {
        self.table.iter().filter_map(|(name, (_, entry, _))| {
            let (vid, id) = entry.virtual_resource()?;
            Some((name.as_str(), vid, id))
        })
    }

    pub fn entries_added(&self) -> impl Iterator<Item = (&str, &ResourceMappingEntry<R, V>)> {
        self.table
            .iter()
            .filter(|(_, (_, _, added))| *added)
            .map(|(name, (_, entry, _))| (name.as_str(), entry))
    }

    pub fn resources_added(&self) -> impl Iterator<Item = (&str, ID<R>)> {
        self.table
            .iter()
            .filter(|(_, (_, _, added))| *added)
            .filter_map(|(name, (_, entry, _))| Some((name.as_str(), entry.resource()?)))
    }

    pub fn virtual_resources_added(&self) -> impl Iterator<Item = (&str, ID<V>, ID<R>)> {
        self.table
            .iter()
            .filter(|(_, (_, _, added))| *added)
            .filter_map(|(name, (_, entry, _))| {
                let (vid, id) = entry.virtual_resource()?;
                Some((name.as_str(), vid, id))
            })
    }

    pub fn removed(&self) -> impl Iterator<Item = &str> {
        self.removed.iter().map(|name| name.as_str())
    }
}
