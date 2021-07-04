use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimalKind {
    GroundWater,
    WaterAir,
    AirGround,
}

impl Prefab for AnimalKind {}
impl PrefabComponent for AnimalKind {}

impl AnimalKind {
    pub fn image(self) -> &'static str {
        match self {
            Self::GroundWater => "images/animal-ground-water.svg",
            Self::WaterAir => "images/animal-water-air.svg",
            Self::AirGround => "images/animal-air-ground.svg",
        }
    }

    pub fn build_image(self, size: Scalar) -> Image<'static> {
        Image::new(self.image())
            .destination(Some([0.0, 0.0, size, size].into()))
            .align([0.5, 0.5].into())
    }

    // TODO: use it in player movement system.
    // pub fn move_cost(self, tile: LevelTile) -> Option<u8> {
    //     match (self, tile) {
    //         (Self::GroundWater, LevelTile::Grass)
    //         | (Self::WaterAir, LevelTile::Pond)
    //         | (Self::AirGround, LevelTile::Lava) => Some(1),
    //         (Self::GroundWater, LevelTile::Pond)
    //         | (Self::WaterAir, LevelTile::Lava)
    //         | (Self::AirGround, LevelTile::Grass) => Some(2),
    //         _ => None,
    //     }
    // }
}
