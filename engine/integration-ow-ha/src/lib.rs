pub mod components;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::{
        components::{board_avatar_sync::*, board_chunk_sync::*, camera_follow_board_entity::*},
        location_to_position,
        resources::*,
        systems::{board_sync::*, camera_follow_board_entity::*},
    };
}

use crate::{
    components::{
        board_avatar_sync::HaBoardAvatarSync, board_chunk_sync::HaBoardChunkSync,
        camera_follow_board_entity::HaCameraFollowBoardEntity,
    },
    resources::HaBoardSettings,
    systems::{
        board_sync::{ha_board_sync_system, HaBoardSyncSystemCache, HaBoardSyncSystemResources},
        camera_follow_board_entity::{
            ha_camera_follow_board_entity, HaCameraFollowBoardEntitySystemResources,
        },
    },
};
use oxygengine_core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    prefab::PrefabManager,
    Scalar,
};
use oxygengine_ha_renderer::math::*;
use oxygengine_overworld::resources::board::{Board, BoardLocation, ChunkLocation, Location};

pub fn location_to_position(location: Location, board: &Board, settings: &HaBoardSettings) -> Vec3 {
    let cell_size = settings.cell_size();
    let origin = settings.origin();
    let (chunk_cols, chunk_rows) = board.chunk_size();
    let dcol = (location.world.col - origin.col) * chunk_cols as isize;
    let drow = (location.world.row - origin.row) * chunk_rows as isize;
    let x = (dcol + location.chunk.col as isize) as Scalar * cell_size.x;
    let y = (drow + location.chunk.row as isize) as Scalar * cell_size.y;
    Vec3::new(x, y, 0.0)
}

pub fn position_to_location(position: Vec3, board: &Board, settings: &HaBoardSettings) -> Location {
    let cell_size = settings.cell_size();
    let origin = settings.origin();
    let (chunk_cols, chunk_rows) = board.chunk_size();
    let shift = Vec3::new(
        origin.col as Scalar * chunk_cols as Scalar * cell_size.x,
        origin.row as Scalar * chunk_rows as Scalar * cell_size.y,
        0.0,
    );
    let position = position - shift;
    let x = (position.x / cell_size.x) as isize;
    let y = (position.y / cell_size.y) as isize;
    let world = BoardLocation::from((
        if x < 0 {
            x / chunk_cols as isize - 1
        } else {
            x / chunk_cols as isize
        },
        if y < 0 {
            y / chunk_rows as isize - 1
        } else {
            y / chunk_rows as isize
        },
    ));
    let chunk = ChunkLocation::from((
        (x - world.col * chunk_cols as isize) as usize,
        (y - world.row * chunk_rows as isize) as usize,
    ));
    (world, chunk).into()
}

pub fn board_chunk_rect(
    location: BoardLocation,
    board: &Board,
    settings: &HaBoardSettings,
) -> Rect {
    let cell_size = settings.cell_size();
    let origin = settings.origin();
    let (chunk_cols, chunk_rows) = board.chunk_size();
    let dcol = (location.col - origin.col) * chunk_cols as isize;
    let drow = (location.row - origin.row) * chunk_rows as isize;
    let x = dcol as Scalar * cell_size.x;
    let y = drow as Scalar * cell_size.y;
    let w = chunk_cols as Scalar * cell_size.x;
    let h = chunk_rows as Scalar * cell_size.y;
    Rect { x, y, w, h }
}

pub fn board_region_rect(
    location_top_left: BoardLocation,
    location_bottom_right: BoardLocation,
    board: &Board,
    settings: &HaBoardSettings,
) -> Rect {
    let cell_size = settings.cell_size();
    let (chunk_cols, chunk_rows) = board.chunk_size();
    let origin = settings.origin();
    let dcol = (location_top_left.col - origin.col) * chunk_cols as isize;
    let drow = (location_top_left.row - origin.row) * chunk_rows as isize;
    let x = (dcol * chunk_cols as isize) as Scalar * cell_size.x;
    let y = (drow * chunk_rows as isize) as Scalar * cell_size.y;
    let w = ((location_bottom_right.col - location_top_left.col + 1) * chunk_cols as isize)
        as Scalar
        * cell_size.x;
    let h = ((location_bottom_right.row - location_top_left.row + 1) * chunk_rows as isize)
        as Scalar
        * cell_size.y;
    Rect { x, y, w, h }
}

pub fn bundle_installer<S, PB>(
    builder: &mut AppBuilder<PB>,
    data: HaBoardSettings,
) -> Result<(), PipelineBuilderError>
where
    S: 'static,
    PB: PipelineBuilder,
{
    builder.install_resource(data);
    builder.install_resource(HaBoardSyncSystemCache::default());

    builder.install_system::<HaBoardSyncSystemResources>(
        "board-sync",
        ha_board_sync_system,
        &["board", "renderer"],
    )?;
    builder.install_system::<HaCameraFollowBoardEntitySystemResources>(
        "camera-follow-board-entity",
        ha_camera_follow_board_entity::<S>,
        &[],
    )?;

    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<HaBoardAvatarSync>("HaBoardAvatarSync");
    prefabs.register_component_factory::<HaBoardChunkSync>("HaBoardChunkSync");
    prefabs.register_component_factory::<HaCameraFollowBoardEntity>("HaCameraFollowBoardEntity");
}
