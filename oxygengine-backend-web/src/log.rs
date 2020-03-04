use core::log::{Log, Logger};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn console_warn(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn console_error(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = debug)]
    pub fn console_debug(s: &str);
}

pub struct WebLogger;

impl Logger for WebLogger {
    fn log(&mut self, mode: Log, message: String) {
        match mode {
            Log::Info => console_log(&format!("[{}] {}", "INFO", message)),
            Log::Warning => console_warn(&format!("[{}] {}", "WARNING", message)),
            Log::Error => console_error(&format!("[{}] {}", "ERROR", message)),
            Log::Debug => console_debug(&format!("[{}] {}", "DEBUG", message)),
        }
    }
}
