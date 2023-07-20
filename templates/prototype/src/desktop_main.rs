mod assets;
mod bootload;
mod character;
mod game;

use crate::{bootload::bootload, game::GameState};
use oxygengine::prelude::*;

pub fn main() {
    bootload(DesktopPrototypeApp::new(GameState::default()))
        .input_mappings(oxygengine::include_input_mappings!(
            "../platforms/desktop/Inputs.toml"
        ))
        .run();
}
