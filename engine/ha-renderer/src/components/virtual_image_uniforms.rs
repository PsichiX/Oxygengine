use crate::{image::ImageFiltering, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVirtualImageUniform {
    pub virtual_asset_name: String,
    #[serde(default)]
    pub filtering: ImageFiltering,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaVirtualImageUniforms {
    /// { uniform name: uniform data }
    #[serde(flatten)]
    data: HashMap<String, HaVirtualImageUniform>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl HaVirtualImageUniforms {
    pub fn get(&self, uniform: &str) -> Option<&HaVirtualImageUniform> {
        self.data.get(uniform)
    }

    pub fn set(&mut self, uniform_name: impl ToString, uniform_data: HaVirtualImageUniform) {
        self.data.insert(uniform_name.to_string(), uniform_data);
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

    pub fn iter(&self) -> impl Iterator<Item = (&str, &HaVirtualImageUniform)> {
        self.data.iter().map(|(k, v)| (k.as_str(), v))
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
