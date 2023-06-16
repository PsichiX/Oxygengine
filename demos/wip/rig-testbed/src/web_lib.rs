mod bootload;
mod states;

use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app = bootload::build_app(
        WebFetchEngine::default(),
        WebStorageEngine,
        make_inputs(),
        WebPlatformInterface::with_canvas_id("screen", WebContextOptions::default())?,
        WebAppTimer::default(),
        |_, _| Ok(()),
    );
    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}

fn make_inputs() -> impl FnMut(&mut InputController) {
    |input| {
        input.register(WebKeyboardInputDevice::new(get_event_target_document()));
        input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
        input.register(WebTouchInputDevice::new(get_event_target_by_id("screen")));
        input.map_config(oxygengine::include_input_mappings!(
            "../platforms/web/Inputs.toml"
        ));
    }
}
