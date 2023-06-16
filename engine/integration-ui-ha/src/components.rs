use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_user_interface::raui::core::layout::CoordsMappingScaling;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaUserInterfaceSync {
    #[serde(default)]
    pub coords_mapping_scaling: CoordsMappingScaling,
    pub colored_material: HaMaterialInstance,
    pub image_material: HaMaterialInstance,
    pub text_material: HaMaterialInstance,
    #[serde(default)]
    pub image_filtering: ImageFiltering,
    #[serde(default)]
    pub text_filtering: ImageFiltering,
}

impl HaUserInterfaceSync {
    pub fn update_references(
        &mut self,
        material_mapping: &MaterialResourceMapping,
        image_mapping: &ImageResourceMapping,
    ) {
        self.colored_material
            .update_references(material_mapping, image_mapping);
        self.image_material
            .update_references(material_mapping, image_mapping);
        self.text_material
            .update_references(material_mapping, image_mapping);
    }
}

impl Prefab for HaUserInterfaceSync {}
impl PrefabComponent for HaUserInterfaceSync {}
