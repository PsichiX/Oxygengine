mod assets;
mod character;
mod game;

use crate::game::GameState;
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() {
    WebPrototypeApp::new(GameState::default())
        .view_size(512.0)
        .sprite_filtering(ImageFiltering::Nearest)
        .preload_asset("image://images/logo.yaml")
        .preload_asset("image://images/panel.yaml")
        .preload_asset("image://images/crab.yaml")
        .preload_asset("font://fonts/roboto.yaml")
        .preload_asset("audio://audio/pop.ogg")
        .run();
}
