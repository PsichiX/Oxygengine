extern crate oxygengine;

#[macro_use]
pub mod macros;

pub mod states;
pub mod systems;

use crate::{states::loading::LoadingState, systems::debug::DebugSystem};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                oxygengine::composite_renderer::protocols_installer(assets);
            }),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"),
                RenderState::new(Some(Color::black())),
            ),
        )
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            input.register(WebMouseInputDevice::new(get_element_by_id("screen")));
            input.map_axis("mouse-x", "mouse", "x");
            input.map_axis("mouse-y", "mouse", "y");
            input.map_trigger("mouse-left", "mouse", "left");
            input.map_trigger("mouse-right", "mouse", "right");
            input.map_trigger("mouse-middle", "mouse", "middle");
        })
        .with_system(DebugSystem, "debug", &[])
        .build(LoadingState, WebAppTimer::default());

    AppRunner::new(app).run::<WebAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
