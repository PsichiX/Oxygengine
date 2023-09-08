use crate::states::loading::LoadingState;
use oxygengine::prelude::{intuicio::prelude::*, *};

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
        .with_bundle(
            oxygengine::core::scripting::bundle_installer,
            make_scripting_registry(),
        )
        .unwrap()
        .with_bundle(oxygengine::input::bundle_installer, inputs_factory)
        .unwrap()
        .with_bundle(
            oxygengine::ha_renderer::bundle_installer,
            make_renderer(renderer_interface),
        )
        .unwrap()
        .with_bundle(extras, ())
        .unwrap()
        .with_resource(storage_engine)
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
    }
}

fn make_scripting_registry() -> Registry {
    let mut registry = Registry::default().with_basic_types();
    oxygengine::core::scripting::scripting_installer(&mut registry);
    oxygengine::ha_renderer::scripting_installer(&mut registry);
    registry
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
                ),
        );

    #[cfg(debug_assertions)]
    renderer.set_error_reporter(LoggerHaRendererErrorReporter);

    HaRendererBundleSetup::new(renderer).with_gizmos(Gizmos::new(HaMaterialInstance::new(
        MaterialReference::Asset("@material/graph/gizmo/color".to_owned()),
    )))
}
