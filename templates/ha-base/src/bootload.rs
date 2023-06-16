use crate::{
    components::{player::*, speed::*},
    states::loading::LoadingState,
    systems::player_movement_system::{player_movement_system, PlayerMovementSystemResources},
};
use oxygengine::prelude::*;

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
        .with_bundle(oxygengine::input::bundle_installer, inputs_factory)
        .unwrap()
        .with_bundle(
            oxygengine::ha_renderer::bundle_installer,
            make_renderer(renderer_interface),
        )
        .unwrap()
        .with_bundle(
            oxygengine::user_interface::bundle_installer::<_, ()>,
            UserInterface::new(crate::ui::setup),
        )
        .unwrap()
        .with_bundle(
            oxygengine::integration_user_interface_ha_renderer::bundle_installer,
            (),
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
        .build::<SequencePipelineEngine, _, _>(LoadingState::default(), app_timer)
}

fn make_assets() -> impl FnMut(&mut AssetsDatabase) {
    |database| {
        #[cfg(debug_assertions)]
        database.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine::ha_renderer::protocols_installer(database);
    }
}

fn make_prefabs() -> impl FnMut(&mut PrefabManager) {
    |prefabs| {
        oxygengine::input::prefabs_installer(prefabs);
        oxygengine::ha_renderer::prefabs_installer(prefabs);
        oxygengine::user_interface::prefabs_installer(prefabs);
        oxygengine::integration_user_interface_ha_renderer::prefabs_installer(prefabs);

        prefabs.register_component_factory::<Player>("Player");
        prefabs.register_component_factory::<Speed>("Speed");
    }
}

fn make_renderer(
    interface: impl HaPlatformInterface + Send + Sync + 'static,
) -> HaRendererBundleSetup {
    let mut renderer = HaRenderer::new(interface)
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
