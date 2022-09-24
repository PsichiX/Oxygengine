use oxygengine::prelude::*;

pub fn input_pointer_to_board_location(
    input: &InputStackListener,
    camera_cache: &CameraCache,
    board: &Board,
    settings: &HaBoardSettings,
) -> Option<Location> {
    if !input
        .trigger_state_or_default("pointer-action")
        .is_pressed()
    {
        return None;
    }

    let point = Vec2::from(input.axes_state_or_default("pointer"));
    camera_cache
        .default_get_first::<RenderPostProcessStage>()
        .map(|info| {
            let point = info.render_target_to_screen(point);
            let point = info.screen_to_world_point(point.into());
            world_position_to_board_location(point, board, settings)
        })
}

pub fn is_touching_side(dx: isize, dy: isize) -> bool {
    matches!((dx, dy), (-1, 0) | (1, 0) | (0, -1) | (0, 1))
}
