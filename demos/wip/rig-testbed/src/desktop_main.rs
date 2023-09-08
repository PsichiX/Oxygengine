#![cfg_attr(feature = "distribution", windows_subsystem = "windows")]

mod bootload;
mod states;

use oxygengine::prelude::*;

pub fn main() {
    #[cfg(debug_assertions)]
    logger_setup(DefaultLogger);

    let runner = DesktopAppRunner::new(DesktopAppConfig::default());
    let app = bootload::build_app(
        FsFetchEngine::default(),
        FsStorageEngine::new("./save"),
        make_inputs(),
        DesktopPlatformInterface::with_context_wrapper(runner.context_wrapper()),
        StandardAppTimer::default(),
        extras,
    );
    AppRunner::new(app).run(runner).unwrap();
}

pub fn extras(
    builder: &mut AppBuilder<LinearPipelineBuilder>,
    _: (),
) -> Result<(), PipelineBuilderError> {
    builder.install_resource(DesktopAppEvents::default());
    Ok(())
}

fn make_inputs() -> impl FnMut(&mut InputController) {
    |input| {
        input.register(DesktopKeyboardInputDevice::default());
        input.register(DesktopMouseInputDevice::default());
        input.map_config(oxygengine::include_input_mappings!(
            "../platforms/desktop/Inputs.toml"
        ));
    }
}
