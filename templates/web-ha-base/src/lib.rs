mod components;
mod states;
mod systems;
mod ui;

use crate::{
    components::{player::*, speed::*},
    states::loading::LoadingState,
    systems::player_movement_system::{player_movement_system, PlayerMovementSystemResources},
};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app = App::build::<LinearPipelineBuilder>()
        .with_bundle(oxygengine::core::assets::bundle_installer, make_assets())
        .unwrap()
        .with_bundle(oxygengine::core::prefab::bundle_installer, make_prefabs())
        .unwrap()
        .with_bundle(oxygengine::input::bundle_installer, make_inputs())
        .unwrap()
        .with_bundle(oxygengine::ha_renderer::bundle_installer, make_renderer()?)
        .unwrap()
        .with_bundle(
            oxygengine::user_interface::bundle_installer::<_, ()>,
            UserInterface::new(ui::setup),
        )
        .unwrap()
        .with_bundle(
            oxygengine::integration_user_interface_ha_renderer::bundle_installer,
            (),
        )
        .unwrap()
        .with_resource(WebStorageEngine)
        .with_system::<PlayerMovementSystemResources>(
            "player-movement",
            player_movement_system,
            &[],
        )
        .unwrap()
        .build::<SequencePipelineEngine, _, _>(LoadingState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}

fn make_assets() -> (WebFetchEngine, impl FnMut(&mut AssetsDatabase)) {
    (WebFetchEngine::default(), |database| {
        #[cfg(debug_assertions)]
        database.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
        oxygengine::ha_renderer::protocols_installer(database);
    })
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

fn make_inputs() -> impl FnMut(&mut InputController) {
    |input| {
        input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
        input.register(WebTouchInputDevice::new(get_event_target_by_id("screen")));
        input.map_trigger("mouse-action", "mouse", "left");
        input.map_axis("w", "keyboard", "KeyW");
        input.map_axis("s", "keyboard", "KeyS");
        input.map_axis("a", "keyboard", "KeyA");
        input.map_axis("d", "keyboard", "KeyD");
        input.map_axis("up", "keyboard", "ArrowUp");
        input.map_axis("down", "keyboard", "ArrowDown");
        input.map_axis("left", "keyboard", "ArrowLeft");
        input.map_axis("right", "keyboard", "ArrowRight");
    }
}

fn make_renderer() -> Result<HaRendererBundleSetup, JsValue> {
    let mut renderer = HaRenderer::new(WebPlatformInterface::with_canvas_id(
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
            MaterialReference::Asset("@material/graph/gizmo/color".to_owned()),
        ))),
    )
}
