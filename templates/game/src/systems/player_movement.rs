use crate::{
    components::{avatar_movement::*, player::*},
    utils::*,
};
use oxygengine::prelude::*;

pub type PlayerMovementSystemResources<'a> = (
    WorldRef,
    &'a InputStack,
    &'a Board,
    &'a HaBoardSettings,
    &'a CameraCache,
    Comp<&'a mut BoardAvatar>,
    Comp<&'a mut HaSpriteAnimationInstance>,
    Comp<&'a AvatarMovement>,
    Comp<&'a Player>,
    Comp<&'a InputStackInstance>,
);

pub fn player_movement_system(universe: &mut Universe) {
    let (world, input_stack, board, settings, camera_cache, ..) =
        universe.query_resources::<PlayerMovementSystemResources>();

    for (_, (avatar, animation, movement, input)) in world
        .query::<(
            &mut BoardAvatar,
            &mut HaSpriteAnimationInstance,
            &AvatarMovement,
            &InputStackInstance,
        )>()
        .with::<&Player>()
        .iter()
    {
        let input = match input_stack.listener_by_instance(input) {
            Some(input) => input,
            None => continue,
        };

        let pointer_board_location =
            input_pointer_to_board_location(input, &camera_cache, &board, &settings);
        let token = match avatar.token() {
            Some(token) => token,
            None => continue,
        };
        animation.set_value("walk", SpriteAnimationValue::Bool(avatar.in_progress()));
        let dir = board_direction_to_vector(avatar.direction().unwrap_or(BoardDirection::South));
        animation.set_value("dir-x", SpriteAnimationValue::Scalar(dir.x));
        animation.set_value("dir-y", SpriteAnimationValue::Scalar(dir.y));

        let (mut from, to) = match (board.token_location(token), pointer_board_location) {
            (Some(from), Some(to)) => (from, to),
            _ => continue,
        };
        avatar.clear_actions_queue();

        let path = match board.find_path(from, to, BoardIgnoreOccupancy::ForTokens(&[token])) {
            Ok(path) => path,
            _ => continue,
        };
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
}
