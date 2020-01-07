use crate::resources::spawner::DespawnEffect;
use oxygengine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Death(pub DespawnEffect);

impl Component for Death {
    type Storage = VecStorage<Self>;
}
