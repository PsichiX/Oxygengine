#[macro_use]
extern crate oxygengine;

use crate::main_state::MainState;
use oxygengine::prelude::*;

mod main_state;

fn main() -> Result<(), ()> {
    // initialize logger to see logs in terminal (debug only).
    #[cfg(debug_assertions)]
    logger_setup(DefaultLogger);

    // Application build phase - install all systems and resources and setup them.
    let app = App::build::<LinearPipelineBuilder>()
        .with_bundle(
            oxygengine::network::bundle_installer::<_, (), DesktopServer>,
            (),
        )
        .unwrap()
        .build::<SequencePipelineEngine, _, _>(MainState::default(), StandardAppTimer::default());

    AppRunner::new(app).run(SyncAppRunner::new())
}
