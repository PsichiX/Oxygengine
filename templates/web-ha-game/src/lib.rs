mod components;
mod model;
mod resources;
mod states;
mod systems;
mod ui;

use crate::{
    components::avatar_movement::AvatarMovement,
    model::item::Item,
    resources::game_state_info::*,
    states::loading::LoadingState,
    systems::avatar_movement::{avatar_movement_system, AvatarMovementSystemResources},
};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

pub const BOARD_TILE_SIZE: (u8, u8) = (8, 8);
pub const BOARD_CHUNK_SIZE: (usize, usize) = (32, 32);
pub const TILE_VALUE_GRASS: usize = 4;
pub const TILE_VALUE_ROAD: usize = 6;
pub const TILE_VALUE_SAND: usize = 7;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app =
        App::build::<LinearPipelineBuilder>()
            .with_bundle(oxygengine::core::assets::bundle_installer, make_assets())
            .unwrap()
            .with_bundle(oxygengine::core::prefab::bundle_installer, make_prefabs())
            .unwrap()
            .with_bundle(oxygengine::input::bundle_installer, make_inputs())
            .unwrap()
            .with_bundle(oxygengine::ha_renderer::bundle_installer, make_renderer()?)
            .unwrap()
            .with_bundle(oxygengine::overworld::bundle_installer, make_overworld())
            .unwrap()
            .with_bundle(
                oxygengine::user_interface::bundle_installer::<_, &GameStateInfo>,
                make_ui(),
            )
            .unwrap()
            .with_bundle(
                oxygengine::integration_user_interface_ha_renderer::bundle_installer,
                (),
            )
            .unwrap()
            .with_bundle(
                oxygengine::integration_overworld_ha_renderer::bundle_installer::<
                    _,
                    RenderForwardStage,
                >,
                make_board_settings(),
            )
            .unwrap()
            .with_resource(WebStorageEngine)
            .with_resource(GameStateInfo::default())
            .with_system::<AvatarMovementSystemResources>(
                "avatar-movement",
                avatar_movement_system,
                &[],
            )
            .unwrap()
            .build::<SequencePipelineEngine, _, _>(LoadingState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}

fn make_assets() -> (WebFetchEngine, impl FnMut(&mut AssetsDatabase)) {
    (WebFetchEngine::default(), |assets| {
        #[cfg(debug_assertions)]
        assets.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine::ha_renderer::protocols_installer(assets);
    })
}

fn make_prefabs() -> impl FnMut(&mut PrefabManager) {
    |prefabs| {
        oxygengine::ha_renderer::prefabs_installer(prefabs);
        oxygengine::user_interface::prefabs_installer(prefabs);
        oxygengine::overworld::prefabs_installer(prefabs);
        oxygengine::integration_user_interface_ha_renderer::prefabs_installer(prefabs);
        oxygengine::integration_overworld_ha_renderer::prefabs_installer(prefabs);

        prefabs.register_component_factory::<AvatarMovement>("AvatarMovement");
    }
}

fn make_inputs() -> impl FnMut(&mut InputController) {
    |input| {
        input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
        input.map_axis("pointer-x", "mouse", "x");
        input.map_axis("pointer-y", "mouse", "y");
        input.map_trigger("pointer-action", "mouse", "left");
        input.map_trigger("pointer-context", "mouse", "right");
        input.map_axis("move-up", "keyboard", "KeyW");
        input.map_axis("move-down", "keyboard", "KeyS");
        input.map_axis("move-left", "keyboard", "KeyA");
        input.map_axis("move-right", "keyboard", "KeyD");
    }
}

fn make_renderer() -> Result<HaRendererBundleSetup, JsValue> {
    Ok(HaRendererBundleSetup::new(
        HaRenderer::new(WebPlatformInterface::with_canvas_id(
            "screen",
            WebContextOptions::default(),
        )?)
        .with_stage::<RenderForwardStage>("forward")
        .with_stage::<RenderGizmoStage>("gizmos")
        .with_stage::<RenderUiStage>("ui")
        .with_pipeline(
            "default",
            PipelineDescriptor::default()
                .render_target("main", RenderTargetDescriptor::Main)
                .stage(
                    StageDescriptor::new("forward")
                        .render_target("main")
                        .domain("@material/domain/surface/flat")
                        .clear_settings(ClearSettings {
                            color: Some(Rgba::gray(0.2)),
                            depth: false,
                            stencil: false,
                        }),
                )
                .stage(
                    StageDescriptor::new("gizmos")
                        .render_target("main")
                        .domain("@material/domain/gizmo"),
                )
                .stage(
                    StageDescriptor::new("ui")
                        .render_target("main")
                        .domain("@material/domain/surface/flat"),
                ),
        ),
    )
    .with_gizmos(Gizmos::new(HaMaterialInstance::new(
        MaterialInstanceReference::Asset("@material/graph/gizmo/color".to_owned()),
    ))))
}

fn make_ui() -> UserInterface {
    UserInterface::new(ui::setup)
        .with_pointer_axis("pointer-x", "pointer-y")
        .with_pointer_trigger("pointer-action", "pointer-context")
}

fn make_overworld() -> impl FnMut(
    &mut Bank<usize>,
    &mut MarketDatabase<Item, usize>,
    &mut QuestsDatabase<(), (), usize>,
) -> Board {
    |_, _, _| {
        Board::new(
            BOARD_CHUNK_SIZE.0,
            BOARD_CHUNK_SIZE.1,
            BoardTraverseRules::default().with_product(&[
                TILE_VALUE_GRASS,
                TILE_VALUE_ROAD,
                TILE_VALUE_SAND,
            ]),
        )
    }
}

fn make_board_settings() -> HaBoardSettings {
    HaBoardSettings::new(Vec2::new(BOARD_TILE_SIZE.0 as _, BOARD_TILE_SIZE.1 as _))
        .with_valid_tile_values([TILE_VALUE_GRASS, TILE_VALUE_ROAD, TILE_VALUE_SAND].into_iter())
        .with_region((0, 0).into(), (1, 1).into())
        .with_region((1, -1).into(), (1, -1).into())
}
