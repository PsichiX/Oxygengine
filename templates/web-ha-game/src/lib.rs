mod components;
mod materials;
mod resources;
mod states;
mod systems;
mod ui;

use crate::{
    components::{
        avatar_combat::AvatarCombat, avatar_movement::AvatarMovement, health::Health,
        weapon::Weapon, Enemy, Player,
    },
    materials::avatar::avatar_material_graph,
    resources::game_state_info::*,
    states::loading::LoadingState,
    systems::{
        avatar_combat::{avatar_combat_system, AvatarCombatSystemResources},
        death::{death_system, DeathSystemResources},
        player_combat::{player_combat_system, PlayerCombatSystemResources},
        player_movement::{player_movement_system, PlayerMovementSystemResources},
        sync_game_state_info::{sync_game_state_info_system, SyncGameStateInfoSystemResources},
    },
};
use oxygengine::prelude::*;
use std::collections::HashMap;
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
            .with_system::<PlayerMovementSystemResources>(
                "player-movement",
                player_movement_system,
                &[],
            )
            .unwrap()
            .with_system::<PlayerCombatSystemResources>("player-combat", player_combat_system, &[])
            .unwrap()
            .with_system::<AvatarCombatSystemResources>(
                "avatar-combat",
                avatar_combat_system,
                &["player-combat"],
            )
            .unwrap()
            .with_system::<DeathSystemResources>("death", death_system, &[])
            .unwrap()
            .with_system::<SyncGameStateInfoSystemResources>(
                "sync-game-state-info",
                sync_game_state_info_system,
                &[],
            )
            .unwrap()
            .with_resource(WebStorageEngine)
            .with_resource(GameStateInfo::default())
            .build::<SequencePipelineEngine, _, _>(LoadingState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}

fn make_assets() -> (WebFetchEngine, impl FnMut(&mut AssetsDatabase)) {
    (WebFetchEngine::default(), |database| {
        #[cfg(debug_assertions)]
        database.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine::ha_renderer::protocols_installer(database);

        database.insert(Asset::new(
            "material",
            "@material/graph/surface/flat/avatar",
            Box::new(MaterialAsset::Graph {
                default_values: {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(
                        "blinkColor".to_owned(),
                        Vec4::new(1.0, 1.0, 1.0, 0.0).into(),
                    );
                    map
                },
                draw_options: MaterialDrawOptions::transparent(),
                content: avatar_material_graph(),
            }),
        ));
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
        prefabs.register_component_factory::<AvatarCombat>("AvatarCombat");
        prefabs.register_component_factory::<Health>("Health");
        prefabs.register_component_factory::<Weapon>("Weapon");
        prefabs.register_component_factory::<Player>("Player");
        prefabs.register_component_factory::<Enemy>("Enemy");
    }
}

fn make_inputs() -> impl FnMut(&mut InputController) {
    |input| {
        input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
        input.map_axis("pointer-x", "mouse", "x");
        input.map_axis("pointer-y", "mouse", "y");
        input.map_trigger("pointer-action", "mouse", "left");
        input.map_axis("move-up", "keyboard", "KeyW");
        input.map_axis("move-down", "keyboard", "KeyS");
        input.map_axis("move-left", "keyboard", "KeyA");
        input.map_axis("move-right", "keyboard", "KeyD");
        input.map_trigger("attack", "keyboard", "Space");
    }
}

fn make_renderer() -> Result<HaRendererBundleSetup, JsValue> {
    let mut renderer = HaRenderer::new(WebPlatformInterface::with_canvas_id(
        "screen",
        WebContextOptions::default(),
    )?)
    .with_stage::<RenderForwardStage>("forward")
    .with_stage::<RenderPostProcessStage>("postprocess")
    .with_stage::<RenderGizmoStage>("gizmos")
    .with_stage::<RenderUiStage>("ui")
    .with_pipeline(
        "default",
        PipelineDescriptor::default()
            .render_target("main", RenderTargetDescriptor::Main)
            .render_target(
                "buffer",
                RenderTargetDescriptor::Custom {
                    buffers: TargetBuffers::default()
                        .with_color(TargetBuffer::color("finalColor"))
                        .map_err(|error| JsValue::from(format!("{:?}", error)))?,
                    width: RenderTargetSizeValue::ScreenAspectHeight {
                        value: 144,
                        round_up: true,
                    },
                    height: RenderTargetSizeValue::ScreenAspectHeight {
                        value: 144,
                        round_up: true,
                    },
                },
            )
            .stage(
                StageDescriptor::new("forward")
                    .render_target("buffer")
                    .domain("@material/domain/surface/flat")
                    .clear_settings(ClearSettings {
                        color: Some(Rgba::gray(0.2)),
                        depth: false,
                        stencil: false,
                    }),
            )
            .stage(
                StageDescriptor::new("postprocess")
                    .render_target("main")
                    .domain("@material/domain/screenspace"),
            )
            .debug_stage(
                StageDescriptor::new("gizmos")
                    .render_target("main")
                    .domain("@material/domain/gizmo"),
            )
            .stage(
                StageDescriptor::new("ui")
                    .render_target("main")
                    .domain("@material/domain/surface/flat"),
            ),
    );

    #[cfg(debug_assertions)]
    renderer.set_error_reporter(LoggerHaRendererErrorReporter);

    Ok(
        HaRendererBundleSetup::new(renderer).with_gizmos(Gizmos::new(HaMaterialInstance::new(
            MaterialInstanceReference::Asset("@material/graph/gizmo/color".to_owned()),
        ))),
    )
}

fn make_ui() -> UserInterface {
    UserInterface::new(ui::setup)
    // .with_pointer_axis("pointer-x", "pointer-y")
    // .with_pointer_trigger("pointer-action", "pointer-context")
}

fn make_overworld() -> impl FnMut(
    &mut Bank<usize>,
    &mut MarketDatabase<(), usize>,
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
