use crate::asset_protocols::part::*;
use oxygengine::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct PartsRegistry {
    drivers: HashMap<String, PartAsset>,
    disks: HashMap<String, PartAsset>,
    layers: HashMap<String, PartAsset>,
    assets: HashMap<AssetId, (String, PartType)>,
}

impl PartsRegistry {
    pub fn register(&mut self, id: AssetId, name: String, part: PartAsset) {
        match part.part_type {
            PartType::Driver => {
                self.assets.insert(id, (name.to_owned(), part.part_type));
                self.drivers.insert(name, part);
            }
            PartType::Disk => {
                self.assets.insert(id, (name.to_owned(), part.part_type));
                self.disks.insert(name, part);
            }
            PartType::Layer => {
                self.assets.insert(id, (name.to_owned(), part.part_type));
                self.layers.insert(name, part);
            }
        }
    }

    pub fn unregister(&mut self, id: AssetId) {
        if let Some((name, part_type)) = self.assets.remove(&id) {
            match part_type {
                PartType::Driver => {
                    self.drivers.remove(&name);
                }
                PartType::Disk => {
                    self.disks.remove(&name);
                }
                PartType::Layer => {
                    self.layers.remove(&name);
                }
            }
        }
    }

    pub fn get(&self, name: &str, part_type: PartType) -> Option<&PartAsset> {
        match part_type {
            PartType::Driver => self.drivers.get(name),
            PartType::Disk => self.disks.get(name),
            PartType::Layer => self.layers.get(name),
        }
    }
}
