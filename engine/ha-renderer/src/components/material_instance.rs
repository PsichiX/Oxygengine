use crate::{
    image::ImageResourceMapping,
    material::{
        common::MaterialValue, MaterialDrawOptions, MaterialReference, MaterialResourceMapping,
    },
};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaMaterialInstance {
    #[serde(default)]
    pub reference: MaterialReference,
    #[serde(default)]
    pub values: HashMap<String, MaterialValue>,
    #[serde(default)]
    pub override_draw_options: Option<MaterialDrawOptions>,
}

impl HaMaterialInstance {
    pub fn new(reference: MaterialReference) -> Self {
        Self {
            reference,
            values: Default::default(),
            override_draw_options: None,
        }
    }

    pub fn with_value(mut self, name: impl ToString, value: MaterialValue) -> Self {
        self.values.insert(name.to_string(), value);
        self
    }

    pub fn with_override_draw_options(mut self, options: MaterialDrawOptions) -> Self {
        self.override_draw_options = Some(options);
        self
    }

    pub fn update_references(
        &mut self,
        material_mapping: &MaterialResourceMapping,
        image_mapping: &ImageResourceMapping,
    ) {
        if let MaterialReference::Asset(path) = &self.reference {
            if let Some(id) = material_mapping.resource_by_name(path) {
                self.reference = MaterialReference::Id(id);
            }
        }
        for value in self.values.values_mut() {
            value.update_references(image_mapping);
        }
    }
}

impl Prefab for HaMaterialInstance {}
impl PrefabComponent for HaMaterialInstance {}
