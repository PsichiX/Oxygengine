use crate::components::material_instance::HaMaterialInstance;
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaPostProcess {
    pub stages: Vec<HaMaterialInstance>,
}

impl Prefab for HaPostProcess {}

impl PrefabComponent for HaPostProcess {}
