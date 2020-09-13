use anim::transition::Transition;
use core::{prefab::Prefab, Ignite, Scalar};
use serde::{Deserialize, Serialize};

pub type BackgroundStyle = Transition<String>;

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Background {
    pub name: String,
    pub image: String,
    #[serde(default = "Background::default_scale")]
    pub scale: Scalar,
}

impl Prefab for Background {}

impl Background {
    fn default_scale() -> Scalar {
        1.0
    }
}
