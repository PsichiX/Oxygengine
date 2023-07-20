mod assets;
mod bootload;
mod character;
mod game;

use crate::{bootload::bootload, game::GameState};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() {
    bootload(WebPrototypeApp::new(GameState::default()))
        .input_mappings(oxygengine::include_input_mappings!(
            "../platforms/web/Inputs.toml"
        ))
        .run();
}
