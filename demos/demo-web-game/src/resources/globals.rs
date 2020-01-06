use oxygengine::prelude::*;

#[derive(Default)]
pub struct Globals {
    pub camera: Option<Entity>,
    pub map_size: Option<Vec2>,
    pub pause: bool,
}

impl Globals {
    pub fn reset(&mut self) {
        self.camera = None;
        self.map_size = None;
        self.pause = false;
    }
}
