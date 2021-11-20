use crate::math::*;
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVirtualImageUniforms {
    /// { uniform name: virtual asset name }
    #[serde(flatten)]
    data: HashMap<String, String>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl HaVirtualImageUniforms {
    pub fn get(&self, uniform: &str) -> Option<&str> {
        self.data.get(uniform).map(|name| name.as_str())
    }

    pub fn set(&mut self, uniform: impl ToString, asset_name: impl ToString) {
        self.data
            .insert(uniform.to_string(), asset_name.to_string());
        self.dirty = true;
    }

    pub fn remove(&mut self, uniform: &str) {
        self.data.remove(uniform);
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.dirty = true;
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.data.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Prefab for HaVirtualImageUniforms {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}

impl PrefabComponent for HaVirtualImageUniforms {}
