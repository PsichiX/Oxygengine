use oxygengine::prelude::*;

#[derive(Default)]
pub struct Typing;

impl Component for Typing {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct History;

impl Component for History {
    type Storage = NullStorage<Self>;
}
