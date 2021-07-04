mod components;
mod states;
mod systems;
mod ui;

use crate::{
    components::{speed::Speed, KeyboardMovementTag},
    states::loading::LoadingState,
    systems::keyboard_movement::{keyboard_movement_system, KeyboardMovementSystemResources},
};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // initialize logger to see logs in web browser (debug only).
    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    // Application build phase - install all systems and resources and setup them.
    let app = App::build::<LinearPipelineBuilder>()
        // install core module assets managment.
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                // register assets loading error reporter that shows errors in console.
                #[cfg(debug_assertions)]
                assets.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
                // register assets protocols from composite renderer module.
                oxygengine::composite_renderer::protocols_installer(assets);
                // register assets protocols from audio module.
                oxygengine::audio::protocols_installer(assets);
            }),
        )
        .unwrap()
        // install core module prefabs management.
        .with_bundle(oxygengine::core::prefab::bundle_installer, |prefabs| {
            // install composite renderer prefabs.
            oxygengine::composite_renderer::prefabs_installer(prefabs);
            // install UI prefabs.
            oxygengine::user_interface::prefabs_installer(prefabs);
            // install prefabs for integration between UI and composite rendering.
            oxygengine::integration_user_interface_composite_renderer::prefabs_installer(prefabs);
            // install audio prefabs.
            oxygengine::audio::prefabs_installer(prefabs);
            // install 2d physics prefabs.
            oxygengine::physics_2d::prefabs_installer(prefabs);
            // install prefabs for integration between 2D physics and composite rendering.
            oxygengine::integration_physics_2d_composite_renderer::prefabs_installer(prefabs);
            // register game prefabs component factories.
            prefabs.register_component_factory::<Speed>("Speed");
            prefabs.register_component_factory::<KeyboardMovementTag>("KeyboardMovementTag");
        })
        .unwrap()
        // install input managment.
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            // register input devices.
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            // map input axes and triggers to devices.
            input.map_axis("pointer-x", "mouse", "x");
            input.map_axis("pointer-y", "mouse", "y");
            input.map_trigger("pointer-action", "mouse", "left");
            input.map_trigger("pointer-context", "mouse", "right");
            input.map_axis("move-up", "keyboard", "KeyW");
            input.map_axis("move-down", "keyboard", "KeyS");
            input.map_axis("move-left", "keyboard", "KeyA");
            input.map_axis("move-right", "keyboard", "KeyD");
        })
        .unwrap()
        // install composite renderer.
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"), // canvas target.
                RenderState::new(Some(Color::rgba(128, 128, 128, 255)))
                    .image_source_inner_margin(0.01),
            ),
        )
        .unwrap()
        // install UI support.
        .with_bundle(
            oxygengine::user_interface::bundle_installer,
            UserInterfaceBundleSetup::default().user_interface(
                UserInterface::new(ui::setup)
                    .with_pointer_axis("pointer-x", "pointer-y")
                    .with_pointer_trigger("pointer-action", "pointer-context"),
            ),
        )
        .unwrap()
        // install integration between UI and composite rendering.
        .with_bundle(
            oxygengine::integration_user_interface_composite_renderer::bundle_installer::<
                WebCompositeRenderer,
                _,
            >,
            (),
        )
        .unwrap()
        // install audio support.
        .with_bundle(oxygengine::audio::bundle_installer, WebAudio::default())
        .unwrap()
        // install 2D physics with default gravity force vector.
        .with_bundle(
            oxygengine::physics_2d::bundle_installer,
            (
                Vector::y() * 9.81 * 7.0,
                Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
            ),
        )
        .unwrap()
        // install integration between 2D physics and composite rendering.
        .with_bundle(
            oxygengine::integration_physics_2d_composite_renderer::bundle_installer,
            (),
        )
        .unwrap()
        // install web storage engine resource.
        .with_resource(WebStorageEngine)
        .with_system::<KeyboardMovementSystemResources>(
            "keyboard-movement",
            keyboard_movement_system,
            &[],
        )
        .unwrap()
        .build::<SequencePipelineEngine, _, _>(LoadingState::default(), WebAppTimer::default());

    // Application run phase - spawn runner that ticks our app.
    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
