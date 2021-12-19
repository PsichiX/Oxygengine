use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Blink(pub Scalar);

impl Prefab for Blink {}
impl PrefabComponent for Blink {}
