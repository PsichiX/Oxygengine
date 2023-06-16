use crate::mesh::{MeshDrawRange, MeshReference, MeshResourceMapping};
use core::prefab::{Prefab, PrefabComponent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaMeshInstance {
    #[serde(default)]
    pub reference: MeshReference,
    #[serde(default)]
    pub override_draw_range: Option<MeshDrawRange>,
}

impl HaMeshInstance {
    pub fn update_references(&mut self, mesh_mapping: &MeshResourceMapping) {
        if let MeshReference::Asset(path) = &self.reference {
            if let Some(id) = mesh_mapping.resource_by_name(path) {
                self.reference = MeshReference::Id(id);
            }
        }
    }
}

impl Prefab for HaMeshInstance {}
impl PrefabComponent for HaMeshInstance {}
