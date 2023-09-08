mod assets;
mod bootload;
mod game;
mod nodes;

use crate::{bootload::bootload, game::GameState};
use oxygengine::prelude::*;

pub fn main() {
    bootload(DesktopPrototypeApp::new_named(
        GameState::default(),
        "Ferris vs Gopher",
    ))
    .input_mappings(oxygengine::include_input_mappings!(
        "../platforms/desktop/Inputs.toml"
    ))
    .run();
}
