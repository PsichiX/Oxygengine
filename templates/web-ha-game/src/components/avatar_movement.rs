use oxygengine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AvatarMovement {
    #[serde(default = "AvatarMovement::default_step_duration")]
    pub step_duration: Scalar,
}

impl AvatarMovement {
    fn default_step_duration() -> Scalar {
        1.0
    }

    pub fn board_direction(v: Vec2) -> Option<BoardDirection> {
        let v = v.try_normalized()?;
        // TODO: change to PI/4.
        let threshold = (45.0 as Scalar).to_radians().cos();
        let dot = Vec2::unit_y().dot(v);
        if dot >= threshold {
            return Some(BoardDirection::South);
        } else if dot <= -threshold {
            return Some(BoardDirection::North);
        }
        let dot = Vec2::unit_x().dot(v);
        if dot >= threshold {
            return Some(BoardDirection::East);
        } else if dot <= -threshold {
            return Some(BoardDirection::West);
        }
        None
    }
}

impl Prefab for AvatarMovement {}

impl PrefabComponent for AvatarMovement {}
