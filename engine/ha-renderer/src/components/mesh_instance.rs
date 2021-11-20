use crate::mesh::{MeshInstanceReference, MeshResourceMapping};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaMeshInstance {
    #[serde(default)]
    pub reference: MeshInstanceReference,
}

impl HaMeshInstance {
    pub fn update_references(&mut self, mesh_mapping: &MeshResourceMapping) {
        if let MeshInstanceReference::Asset(path) = &self.reference {
            if let Some(id) = mesh_mapping.resource_by_name(path) {
                self.reference = MeshInstanceReference::Id(id);
            }
        }
    }
}

impl Prefab for HaMeshInstance {}

impl PrefabComponent for HaMeshInstance {}
