use crate::components::{avatar_movement::*, *};
use oxygengine::prelude::*;

pub type PlayerMovementSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    &'a Board,
    &'a HaBoardSettings,
    &'a CameraCache,
    Comp<&'a mut BoardAvatar>,
    Comp<&'a mut HaSpriteAnimationInstance>,
    Comp<&'a AvatarMovement>,
    Comp<&'a Player>,
);

pub fn player_movement_system(universe: &mut Universe) {
    let (world, input, board, settings, camera_cache, ..) =
        universe.query_resources::<PlayerMovementSystemResources>();

    let pointer_x = input.axis_or_default("pointer-x");
    let pointer_y = input.axis_or_default("pointer-y");
    let pointer_action = input.trigger_or_default("pointer-action").is_pressed();
    let target_location = if pointer_action {
        camera_cache
            .default_get_first::<RenderForwardStage>()
            .map(|info| {
                let point = info.render_target_to_screen(Vec2::new(pointer_x, pointer_y));
                let point = info.screen_to_world_point(point.into());
                world_position_to_board_location(point, &board, &settings)
            })
    } else {
        None
    };
    let dir_x = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
    let dir_y = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
    let direction = vector_to_board_direction(Vec2::new(dir_x, dir_y), false);

    for (_, (avatar, animation, movement)) in world
        .query::<(
            &mut BoardAvatar,
            &mut HaSpriteAnimationInstance,
            &AvatarMovement,
        )>()
        .with::<Player>()
        .iter()
    {
        if let Some(token) = avatar.token() {
            animation.set_value("walk", SpriteAnimationValue::Bool(avatar.in_progress()));
            let dir =
                board_direction_to_vector(avatar.direction().unwrap_or(BoardDirection::South));
            animation.set_value("dir-x", SpriteAnimationValue::Scalar(dir.x));
            animation.set_value("dir-y", SpriteAnimationValue::Scalar(dir.y));
            if let (Some(mut from), Some(to)) = (board.token_location(token), target_location) {
                avatar.clear_actions_queue();
                if let Ok(path) =
                    board.find_path(from, to, BoardIgnoreOccupancy::ForTokens(&[token]))
                {
                    for location in path.into_iter().skip(1) {
                        let (x, y) = board.location_relative(from, location);
                        from = location;
                        avatar.enqueue_action(BoardAvatarAction::Move {
                            duration: movement.step_duration,
                            x,
                            y,
                        });
                    }
                }
            } else if let Some(direction) = direction {
                if !avatar.in_progress() {
                    avatar.perform_single_action(BoardAvatarAction::MoveStep {
                        duration: movement.step_duration,
                        direction,
                    });
                }
            }
        }
    }
}
