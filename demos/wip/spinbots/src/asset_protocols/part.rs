use crate::utils::spinbot::*;
use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartType {
    Driver,
    Disk,
    Layer,
}

impl Default for PartType {
    fn default() -> Self {
        Self::Driver
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct PartAsset {
    pub name: String,
    pub part_type: PartType,
    pub radius: Scalar,
    pub mass: Scalar,
    #[serde(default)]
    pub linear_damping: Scalar,
    #[serde(default)]
    pub angular_damping: Scalar,
    #[serde(default)]
    pub friction_coefficient: Scalar,
    #[serde(default)]
    pub restitution_coefficient: Scalar,
}

impl PartAsset {
    pub fn stats(&self) -> SpinBotStats {
        SpinBotStats {
            radius: self.radius,
            mass: self.mass,
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            friction_coefficient: self.friction_coefficient,
            restitution_coefficient: self.restitution_coefficient,
        }
    }
}

pub struct PartAssetProtocol;

impl AssetProtocol for PartAssetProtocol {
    fn name(&self) -> &str {
        "part"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        let config = serde_yaml::from_str::<PartAsset>(data).unwrap();
        AssetLoadResult::Data(Box::new(config))
    }
}
