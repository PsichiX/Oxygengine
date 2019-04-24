extern crate oxygengine;

#[macro_use]
pub mod macros;

pub mod states;
pub mod systems;

use crate::states::loading::LoadingState;
use oxygengine::{
    composite_renderer::{composite_renderer::*, math::*},
    prelude::*,
};
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
            (PlatformFetchEngine::default(), |assets| {
                oxygengine::composite_renderer::protocols_installer(assets);
            }),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            PlatformCompositeRenderer::with_state("screen", RenderState::new(Some(Color::black()))),
        )
        // .with_system(DebugSystem, "debug", &[])
        .build(LoadingState, PlatformAppTimer::default());

    AppRunner::new(app).run::<PlatformAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
