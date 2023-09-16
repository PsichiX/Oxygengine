pub mod control;
pub mod deformer;
pub mod skeleton;

use crate::mesh::rig::{control::RigControl, deformer::Deformer, skeleton::Skeleton};

#[derive(Debug, Default, Clone)]
pub struct Rig {
    pub skeleton: Skeleton,
    pub deformer: Deformer,
    pub controls: Vec<RigControl>,
}
