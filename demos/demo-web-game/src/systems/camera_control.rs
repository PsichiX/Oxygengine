#![allow(clippy::type_complexity)]

use crate::resources::globals::Globals;
use oxygengine::prelude::*;

pub struct CameraControlSystem;

impl<'s> System<'s> for CameraControlSystem {
    type SystemData = (
        ReadExpect<'s, WebCompositeRenderer>,
        Read<'s, Globals>,
        ReadStorage<'s, CompositeCamera>,
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(&mut self, (renderer, globals, cameras, mut transforms): Self::SystemData) {
        if globals.camera.is_none() || globals.map_size.is_none() {
            return;
        }
        let entity = globals.camera.unwrap();
        let map_size = globals.map_size.unwrap();
        let screen_size = renderer.view_size();
        let view_box = if let Some(transform) = transforms.get(entity) {
            if let Some(camera) = cameras.get(entity) {
                camera.view_box(transform, screen_size)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(mut view_box) = view_box {
            view_box.x = view_box.x.max(0.0).min(map_size.x - view_box.w);
            view_box.y = view_box.y.max(0.0).min(map_size.y - view_box.h);
            transforms.get_mut(entity).unwrap().set_translation(view_box.center());
        }
    }
}
