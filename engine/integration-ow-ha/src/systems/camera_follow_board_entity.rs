use crate::{
    board_chunk_rect, board_region_rect,
    components::camera_follow_board_entity::{
        HaCameraFollowBoardEntity, HaCameraFollowConstraints,
    },
    resources::HaBoardSettings,
    world_position_to_board_location,
};
use oxygengine_core::{
    app::AppLifeCycle,
    ecs::{hierarchy::Hierarchy, Comp, Universe, WorldRef},
    Scalar,
};
use oxygengine_ha_renderer::{
    components::{camera::HaCamera, transform::HaTransform},
    math::*,
    resources::camera_cache::CameraCache,
};
use oxygengine_overworld::resources::board::Board;

pub type HaCameraFollowBoardEntitySystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a Hierarchy,
    &'a Board,
    &'a HaBoardSettings,
    &'a CameraCache,
    Comp<&'a mut HaTransform>,
    Comp<&'a HaCamera>,
    Comp<&'a HaCameraFollowBoardEntity>,
);

pub fn ha_camera_follow_board_entity<T: 'static>(universe: &mut Universe) {
    let (world, lifecycle, hierarchy, board, settings, cache, ..) =
        universe.query_resources::<HaCameraFollowBoardEntitySystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for (me, (my_transform, follow)) in world
        .query::<(&mut HaTransform, &HaCameraFollowBoardEntity)>()
        .with::<HaCamera>()
        .iter()
    {
        if let Some(other) = follow
            .name
            .as_ref()
            .and_then(|n| hierarchy.find(None, n.as_str()))
        {
            if me != other {
                if let Ok(other_transform) = world.get::<HaTransform>(other) {
                    let from = my_transform.get_translation();
                    let mut to = other_transform.get_translation();
                    let f = (follow.strength_factor * dt).max(0.0).min(1.0);
                    match follow.constraints {
                        HaCameraFollowConstraints::None => {
                            my_transform.set_translation(Vec3::lerp(from, to, f));
                        }
                        HaCameraFollowConstraints::Chunk => {
                            let (min, max) = match cache
                                .get::<T>(me, follow.nth)
                                .map(|info| info.world_bounds())
                            {
                                Some(result) => result,
                                None => continue,
                            };
                            let location = world_position_to_board_location(to, &board, &settings);
                            let rect = board_chunk_rect(location.world, &board, &settings);
                            let width = max.x - min.x;
                            let height = max.y - min.y;
                            to = fit_position_in_box(to, rect, width, height);
                            my_transform.set_translation(Vec3::lerp(from, to, f));
                        }
                        HaCameraFollowConstraints::Region => {
                            let (min, max) = match cache
                                .get::<T>(me, follow.nth)
                                .map(|info| info.world_bounds())
                            {
                                Some(result) => result,
                                None => continue,
                            };
                            let location = world_position_to_board_location(to, &board, &settings);
                            let (top_left, bottom_right) =
                                match settings.find_region(location.world) {
                                    Some(result) => result,
                                    None => continue,
                                };
                            let rect = board_region_rect(top_left, bottom_right, &board, &settings);
                            let width = max.x - min.x;
                            let height = max.y - min.y;
                            to = fit_position_in_box(to, rect, width, height);
                            my_transform.set_translation(Vec3::lerp(from, to, f));
                        }
                    }
                }
            }
        }
    }
}

fn fit_position_in_box(mut position: Vec3, rect: Rect, width: Scalar, height: Scalar) -> Vec3 {
    let half_width = width * 0.5;
    let half_height = height * 0.5;
    if width > rect.w {
        position.x = rect.x + rect.w * 0.5;
    } else {
        position.x = position
            .x
            .max(rect.x + half_width)
            .min(rect.x + rect.w - half_width);
    }
    if height > rect.h {
        position.y = rect.y + rect.h * 0.5;
    } else {
        position.y = position
            .y
            .max(rect.y + half_height)
            .min(rect.y + rect.h - half_height);
    }
    position
}
