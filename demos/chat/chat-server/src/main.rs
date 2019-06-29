#[macro_use]
extern crate oxygengine;

mod consts;
mod main_state;

use main_state::MainState;
use oxygengine::prelude::*;

fn main() -> Result<(), ()> {
    logger_setup(DefaultLogger);

    let app = App::build()
        .with_bundle(
            oxygengine::network::bundle_installer::<(), DesktopServer>,
            (),
        )
        .build(MainState::new(10), StandardAppTimer::default());

    AppRunner::new(app).run(SyncAppRunner::new())
}
