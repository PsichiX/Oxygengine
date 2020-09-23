#[macro_use]
extern crate oxygengine;

mod components;
mod resource;
mod states;
mod systems;

use crate::{
    components::{speed::Speed, KeyboardMovementTag},
    resource::text_inputs::TextInputs,
    states::loading::LoadingState,
    systems::{
        keyboard_movement::KeyboardMovementSystem,
        text_input::{TextInputRendererSystem, TextInputWriterSystem},
    },
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
    let app = App::build()
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
        // install core module prefabs management.
        .with_bundle(oxygengine::core::prefab::bundle_installer, |prefabs| {
            // install composite renderer prefabs.
            oxygengine::composite_renderer::prefabs_installer(prefabs);
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
        // install input managment.
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            // register input devices.
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            // map input axes and triggers to devices.
            input.map_axis("move-up", "keyboard", "KeyW");
            input.map_axis("move-down", "keyboard", "KeyS");
            input.map_axis("move-left", "keyboard", "KeyA");
            input.map_axis("move-right", "keyboard", "KeyD");
            input.map_axis("mouse-x", "mouse", "x");
            input.map_axis("mouse-y", "mouse", "y");
            input.map_trigger("mouse-left", "mouse", "left");
        })
        // install composite renderer.
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"), // canvas target.
                RenderState::new(Some(Color::black())),
            ),
        )
        // install audio support.
        .with_bundle(oxygengine::audio::bundle_installer, WebAudio::default())
        // install 2D physics with default gravity force vector.
        .with_bundle(
            oxygengine::physics_2d::bundle_installer,
            (
                Vector::y() * 9.81 * 7.0,
                Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
            ),
        )
        // install integration between 2D physics and composite rendering.
        .with_bundle(
            oxygengine::integration_physics_2d_composite_renderer::bundle_installer,
            (),
        )
        // install web storage engine resource.
        .with_resource(WebStorageEngine)
        .with_resource(TextInputs::default())
        .with_system(KeyboardMovementSystem, "keyboard_movement", &[])
        .with_system(TextInputWriterSystem, "text_input_writer", &[])
        .with_system(
            TextInputRendererSystem::default(),
            "text_input_renderer",
            &[],
        )
        .build(LoadingState::default(), WebAppTimer::default());

    // Application run phase - spawn runner that ticks our app.
    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
