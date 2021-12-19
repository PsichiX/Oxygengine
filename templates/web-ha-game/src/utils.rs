use oxygengine::prelude::*;

pub fn input_pointer_to_board_location(
    input: &InputController,
    camera_cache: &CameraCache,
    board: &Board,
    settings: &HaBoardSettings,
) -> Option<Location> {
    let point = if input.trigger_or_default("touch-action").is_pressed() {
        Some(Vec2::new(
            input.axis_or_default("touch-x"),
            input.axis_or_default("touch-y"),
        ))
    } else if input.trigger_or_default("pointer-action").is_pressed() {
        Some(Vec2::new(
            input.axis_or_default("pointer-x"),
            input.axis_or_default("pointer-y"),
        ))
    } else {
        None
    };
    if let Some(point) = point {
        camera_cache
            .default_get_first::<RenderPostProcessStage>()
            .map(|info| {
                let point = info.render_target_to_screen(point);
                let point = info.screen_to_world_point(point.into());
                world_position_to_board_location(point, board, settings)
            })
    } else {
        None
    }
}

pub fn is_touching_side(dx: isize, dy: isize) -> bool {
    matches!((dx, dy), (-1, 0) | (1, 0) | (0, -1) | (0, 1))
}
