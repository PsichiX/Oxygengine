use crate::{
    components::{
        avatar_combat::AvatarCombat, avatar_movement::AvatarMovement, blink::Blink, enemy::Enemy,
        health::Health, level_up::LevelUp, player::Player, weapon::Weapon, BatchedAttacksTag,
        BatchedSecretsTag,
    },
    materials::avatar::{avatar_material_graph, virtual_uniform_avatar_material_graph},
    resources::{effects::*, game_state_info::*, GlobalEvent},
    states::loading::LoadingState,
    systems::{
        area::{area_system, AreaSystemResources},
        avatar_combat::{avatar_combat_system, AvatarCombatSystemResources},
        blink::{blink_system, BlinkSystemResources},
        death::{death_system, DeathSystemResources},
        effects::{effects_system, EffectsSystemResources},
        global_events::{global_events_system, GlobalEventsSystemResources},
        player_collect_secrets::{
            player_collect_secrets_system, PlayerCollectSecretsSystemResources,
        },
        player_combat::{player_combat_system, PlayerCombatSystemResources},
        player_movement::{player_movement_system, PlayerMovementSystemResources},
        sync_game_state_info::{sync_game_state_info_system, SyncGameStateInfoSystemResources},
    },
};
use oxygengine::prelude::*;
use std::collections::HashMap;

pub const BOARD_TILE_SIZE: (u8, u8) = (8, 8);
pub const BOARD_CHUNK_SIZE: (usize, usize) = (32, 32);
pub const TILE_VALUE_GRASS: usize = 4;
pub const TILE_VALUE_ROAD: usize = 6;
pub const TILE_VALUE_SAND: usize = 7;
pub const TILE_VALUE_BUILDING: usize = 8;

pub fn build_app(
    fetch_engine: impl FetchEngine + 'static,
    storage_engine: impl StorageEngine + 'static,
    inputs_factory: impl FnMut(&mut InputController),
    renderer_interface: impl HaPlatformInterface + Send + Sync + 'static,
    app_timer: impl AppTimer + 'static,
    extras: impl FnMut(&mut AppBuilder<LinearPipelineBuilder>, ()) -> Result<(), PipelineBuilderError>,
) -> App {
    App::build::<LinearPipelineBuilder>()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (fetch_engine, make_assets()),
        )
        .unwrap()
        .with_bundle(oxygengine::core::prefab::bundle_installer, make_prefabs())
        .unwrap()
        .with_bundle(oxygengine::core::ecs::life_cycle::events_system_installer::<_, HaVolumeOverlapEvent>, "volume-overlap")
        .unwrap()
        .with_bundle(oxygengine::input::bundle_installer, inputs_factory)
        .unwrap()
        .with_bundle(
            oxygengine::ha_renderer::bundle_installer,
            make_renderer(renderer_interface),
        )
        .unwrap()
        .with_bundle(
            oxygengine::ha_renderer::immediate_batch_system_installer::<_, SurfaceVertexPT>,
            "pt",
        )
        .unwrap()
        .with_bundle(oxygengine::overworld::bundle_installer, make_overworld())
        .unwrap()
        .with_bundle(
            oxygengine::user_interface::bundle_installer::<_, &GameStateInfo>,
            UserInterface::new(crate::ui::setup),
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
        .with_bundle(extras, ())
        .unwrap()
        .with_resource(storage_engine)
        .with_system::<PlayerMovementSystemResources>(
            "player-movement",
            player_movement_system,
            &[],
        )
        .unwrap()
        .with_system::<PlayerCombatSystemResources>("player-combat", player_combat_system, &[])
        .unwrap()
        .with_system::<PlayerCollectSecretsSystemResources>(
            "player-collect-secrets",
            player_collect_secrets_system,
            &[],
        )
        .unwrap()
        .with_system::<AvatarCombatSystemResources>(
            "avatar-combat",
            avatar_combat_system,
            &["player-combat"],
        )
        .unwrap()
        .with_system::<DeathSystemResources>("death", death_system, &[])
        .unwrap()
        .with_system::<BlinkSystemResources>("blink", blink_system, &[])
        .unwrap()
        .with_system::<AreaSystemResources>("area", area_system, &[])
        .unwrap()
        .with_system::<EffectsSystemResources>("effects", effects_system, &[])
        .unwrap()
        .with_system::<GlobalEventsSystemResources>("global-events", global_events_system, &[])
        .unwrap()
        .with_system::<SyncGameStateInfoSystemResources>(
            "sync-game-state-info",
            sync_game_state_info_system,
            &[],
        )
        .unwrap()
        .with_resource(GameStateInfo::default())
        .with_resource(Effects::with_capacity(10, 10))
        .with_resource(Events::<GlobalEvent>::default())
        .build::<SequencePipelineEngine, _, _>(LoadingState::default(), app_timer)
}

fn make_assets() -> impl FnMut(&mut AssetsDatabase) {
    |database| {
        #[cfg(debug_assertions)]
        database.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine::ha_renderer::protocols_installer(database);

        database.insert(Asset::new(
            "material",
            "@material/graph/surface/flat/avatar",
            MaterialAsset::Graph {
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
            },
        ));

        database.insert(Asset::new(
            "material",
            "@material/graph/surface/flat/virtual-uniform-avatar",
            MaterialAsset::Graph {
                default_values: {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(
                        "blinkColor".to_owned(),
                        Vec4::new(1.0, 1.0, 1.0, 0.0).into(),
                    );
                    map
                },
                draw_options: MaterialDrawOptions::transparent(),
                content: virtual_uniform_avatar_material_graph(),
            },
        ));
    }
}

fn make_prefabs() -> impl FnMut(&mut PrefabManager) {
    |prefabs| {
        oxygengine::core::ecs::life_cycle::events_prefab_installer::<HaVolumeOverlapEvent>(
            "HaVolumeOverlapEvent",
            prefabs,
        );
        oxygengine::input::prefabs_installer(prefabs);
        oxygengine::ha_renderer::prefabs_installer(prefabs);
        oxygengine::ha_renderer::immediate_batch_prefab_installer::<SurfaceVertexPT>("PT", prefabs);
        oxygengine::user_interface::prefabs_installer(prefabs);
        oxygengine::overworld::prefabs_installer(prefabs);
        oxygengine::integration_user_interface_ha_renderer::prefabs_installer(prefabs);
        oxygengine::integration_overworld_ha_renderer::prefabs_installer(prefabs);

        prefabs.register_component_factory::<AvatarMovement>("AvatarMovement");
        prefabs.register_component_factory::<AvatarCombat>("AvatarCombat");
        prefabs.register_component_factory::<Blink>("Blink");
        prefabs.register_component_factory::<Enemy>("Enemy");
        prefabs.register_component_factory::<Health>("Health");
        prefabs.register_component_factory::<Player>("Player");
        prefabs.register_component_factory::<Weapon>("Weapon");
        prefabs.register_component_factory::<BatchedSecretsTag>("BatchedSecretsTag");
        prefabs.register_component_factory::<BatchedAttacksTag>("BatchedAttacksTag");
        prefabs.register_component_factory::<LevelUp>("LevelUp");
    }
}

fn make_renderer(
    interface: impl HaPlatformInterface + Send + Sync + 'static,
) -> HaRendererBundleSetup {
    let mut renderer = HaRenderer::new(interface)
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
                            .unwrap_or_else(|error| panic!("{:?}", error)),
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

    HaRendererBundleSetup::new(renderer).with_gizmos(Gizmos::new(HaMaterialInstance::new(
        MaterialReference::Asset("@material/graph/gizmo/color".to_owned()),
    )))
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
        .with_valid_tile_values(
            [
                TILE_VALUE_GRASS,
                TILE_VALUE_ROAD,
                TILE_VALUE_SAND,
                TILE_VALUE_BUILDING,
            ]
            .into_iter(),
        )
        .with_region((0, 0).into(), (1, 1).into())
        .with_region((1, -1).into(), (1, -1).into())
}
