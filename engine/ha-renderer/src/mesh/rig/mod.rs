pub mod deformer;
pub mod skeleton;

use crate::mesh::rig::{deformer::Deformer, skeleton::Skeleton};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Rig {
    #[serde(default)]
    pub skeleton: Skeleton,
    #[serde(default)]
    pub deformer: Deformer,
}
