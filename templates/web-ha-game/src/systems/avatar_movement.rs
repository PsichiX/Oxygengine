use crate::components::avatar_movement::*;
use oxygengine::prelude::*;

pub type AvatarMovementSystemResources<'a> = (
    WorldRef,
    &'a InputController,
    Comp<&'a mut BoardAvatar>,
    Comp<&'a mut HaSpriteAnimationInstance>,
    Comp<&'a AvatarMovement>,
);

pub fn avatar_movement_system(universe: &mut Universe) {
    let (world, input, ..) = universe.query_resources::<AvatarMovementSystemResources>();

    let x = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
    let y = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
    let direction = AvatarMovement::board_direction(Vec2::new(x, y));

    for (_, (avatar, animation, movement)) in world
        .query::<(
            &mut BoardAvatar,
            &mut HaSpriteAnimationInstance,
            &AvatarMovement,
        )>()
        .iter()
    {
        if avatar.has_token() {
            animation.set_value("walk", SpriteAnimationValue::Bool(avatar.in_progress()));
            if let Some(direction) = direction {
                animation.set_value("dir-x", SpriteAnimationValue::Scalar(x));
                animation.set_value("dir-y", SpriteAnimationValue::Scalar(y));
                avatar.perform_single_action(BoardAvatarAction::MoveStep {
                    duration: movement.step_duration,
                    direction,
                });
            } else {
                avatar.clear_actions_queue();
            }
        }
    }
}
