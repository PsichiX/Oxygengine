use oxygengine::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct FollowMouseTag;

impl Component for FollowMouseTag {
    type Storage = NullStorage<Self>;
}
