use crate::states::loading::LoadingState;
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

mod components;
mod states;
mod systems;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                #[cfg(debug_assertions)]
                assets.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
                oxygengine::composite_renderer::protocols_installer(assets);
            }),
        )
        .with_bundle(oxygengine::core::prefab::bundle_installer, |prefabs| {
            oxygengine::composite_renderer::prefabs_installer(prefabs);
        })
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            input.map_axis("move-up", "keyboard", "KeyW");
            input.map_axis("move-down", "keyboard", "KeyS");
            input.map_axis("move-left", "keyboard", "KeyA");
            input.map_axis("move-right", "keyboard", "KeyD");
            input.map_axis("mouse-x", "mouse", "x");
            input.map_axis("mouse-y", "mouse", "y");
            input.map_trigger("mouse-left", "mouse", "left");
        })
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"),
                RenderState::new(Some(Color::black())),
            ),
        )
        .build(LoadingState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
