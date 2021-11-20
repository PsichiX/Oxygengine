use wasm_bindgen::JsCast;
use web_sys::*;

pub fn get_event_target_by_id(id: &str) -> EventTarget {
    let document = window().document().expect("no `window.document` exists");
    let canvas = document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("no `{}` event target in document", id));
    canvas.dyn_into::<EventTarget>().map_err(|_| ()).unwrap()
}

pub fn get_event_target_document() -> EventTarget {
    let document = window().document().expect("no `window.document` exists");
    document.dyn_into::<EventTarget>().map_err(|_| ()).unwrap()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}
