#[macro_use]
extern crate oxygengine;

mod components;
mod consts;
mod resources;
mod states;
mod systems;

use crate::{
    states::main_state::MainState,
    systems::{
        history_system::HistorySystem, text_input_system::TextInputSystem,
        typing_system::TypingSystem,
    },
};
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
            }),
        )
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        })
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"),
                RenderState::new(Some(Color::black())),
            ),
        )
        .with_bundle(oxygengine::network::bundle_installer::<WebClient, ()>, ())
        .with_system(TextInputSystem, "text_input", &[])
        .with_system(TypingSystem, "typing", &["text_input"])
        .with_system(HistorySystem, "history", &[])
        .build(MainState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
