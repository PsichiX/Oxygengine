use crate::components::FollowMouseTag;
use oxygengine::prelude::*;

pub struct FollowMouseSystem;

impl<'s> System<'s> for FollowMouseSystem {
    type SystemData = (
        Read<'s, InputController>,
        Read<'s, CompositeTransformRes>,
        ReadStorage<'s, Tag>,
        ReadStorage<'s, FollowMouseTag>,
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(
        &mut self,
        (input, transform_res, tags, follow_mouse, mut transforms): Self::SystemData,
    ) {
        let screen_pos = Vec2::new(
            input.axis_or_default("mouse-x"),
            input.axis_or_default("mouse-y"),
        );
        // let camera_inv_matrix = (&tags, transform_res.read_inverse())
        //     .join()
        //     .find_map(|(tag, matrix)| {
        //         if tag.0 == "camera" {
        //             Some(*matrix)
        //         } else {
        //             None
        //         }
        //     })
        //     .unwrap_or_else(|| ind_mat());
        // let global_pos = screen_pos * camera_inv_matrix;

        for (_, transform) in (&follow_mouse, &mut transforms).join() {
            transform.set_translation(screen_pos);
        }
    }
}
