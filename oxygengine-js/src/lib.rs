use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                oxygengine::composite_renderer::protocols_installer(assets);
                oxygengine::audio::protocols_installer(assets);
                oxygengine::script::web::protocols_installer(assets);
            }),
        )
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
        })
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"),
                RenderState::new(Some(Color::black())),
            ),
        )
        .with_bundle(oxygengine::audio::bundle_installer, WebAudio::default())
        // .with_bundle(
        //     oxygengine::physics_2d::bundle_installer,
        //     (
        //         Vector::y() * 9.81,
        //         Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
        //     ),
        // )
        // .with_bundle(
        //     oxygengine::integration_physics_2d_composite_renderer::bundle_installer,
        //     (),
        // )
        .with_bundle(oxygengine::script::web::bundle_installer, |_| {})
        .build(WebScriptBootState::new("main"), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
