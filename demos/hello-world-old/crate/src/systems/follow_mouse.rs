use crate::components::FollowMouseTag;
use oxygengine::prelude::*;

pub struct FollowMouseSystem;

impl<'s> System<'s> for FollowMouseSystem {
    type SystemData = (
        Read<'s, InputController>,
        ReadExpect<'s, WebCompositeRenderer>,
        ReadStorage<'s, Name>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, FollowMouseTag>,
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(
        &mut self,
        (input, renderer, names, cameras, follow_mouse, mut transforms): Self::SystemData,
    ) {
        let view_size = renderer.view_size();
        let screen_pos = Vec2::new(
            input.axis_or_default("mouse-x"),
            input.axis_or_default("mouse-y"),
        );
        if let Some(camera_inv_view) =
            (&names, &cameras, &transforms)
                .join()
                .find_map(|(name, camera, transform)| {
                    if name.0 == "main-camera" {
                        !camera.view_matrix(transform, view_size)
                    } else {
                        None
                    }
                })
        {
            let global_pos = screen_pos * camera_inv_view;

            for (_, transform) in (&follow_mouse, &mut transforms).join() {
                transform.set_translation(global_pos);
            }
        }
    }
}
