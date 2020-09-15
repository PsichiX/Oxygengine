use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Serialize, Deserialize)]
struct MetaScreenshot {
    pub id: String,
    #[serde(default)]
    pub preview: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MetaScreenshotResponse {
    pub id: String,
    pub data: String,
    #[serde(default)]
    pub preview: bool,
}

#[cfg(feature = "composite-renderer")]
pub fn ipc_on_screenshot(
    canvas: web_sys::HtmlCanvasElement,
) -> Box<oxygengine_backend_web::app::WebIpcCallback> {
    Box::new(move |data, _origin| {
        if let Ok(meta) = data.into_serde::<MetaScreenshot>() {
            if meta.id == "screenshot" {
                if let Ok(data) = canvas.to_data_url() {
                    if let Ok(result) = JsValue::from_serde(&MetaScreenshotResponse {
                        id: meta.id,
                        data,
                        preview: meta.preview,
                    }) {
                        return Some(result);
                    }
                }
            }
        }
        None
    })
}
