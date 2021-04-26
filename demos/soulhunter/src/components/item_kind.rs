use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Star,
    Shield,
}

impl Component for ItemKind {
    type Storage = VecStorage<Self>;
}

impl Prefab for ItemKind {}
impl PrefabComponent for ItemKind {}

impl ItemKind {
    pub fn image(self) -> &'static str {
        match self {
            Self::Star => "images/item-star.svg",
            Self::Shield => "images/item-shield.svg",
        }
    }

    pub fn build_image(self, size: Scalar) -> Image<'static> {
        Image::new(self.image())
            .destination(Some([0.0, 0.0, size, size].into()))
            .align([0.5, 0.5].into())
    }
}
