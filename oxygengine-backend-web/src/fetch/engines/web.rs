use core::fetch::{FetchCancelReason, FetchEngine, FetchProcess, FetchProcessReader, FetchStatus};
use futures::{future, Future};
use js_sys::{ArrayBuffer, Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

#[derive(Default, Clone)]
pub struct WebFetchEngine {
    root_path: String,
}

impl WebFetchEngine {
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_owned(),
        }
    }
}

impl FetchEngine for WebFetchEngine {
    fn fetch(&mut self, path: &str) -> Result<Box<FetchProcessReader>, FetchStatus> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        console_log!("=== FETCH | begin");
        let full_path = format!("{}/{}", self.root_path, path);
        console_log!("=== FETCH | file: {:?}", full_path);
        let request = Request::new_with_str_and_init(&full_path, &opts).unwrap();
        let request_promise = window().fetch_with_request(&request);
        let process = FetchProcess::new_start();
        let mut process2 = process.clone();
        console_log!("=== FETCH | build future");
        let future = JsFuture::from(request_promise)
            .and_then(|resp| {
                console_log!("=== FETCH | resp");
                assert!(resp.is_instance_of::<Response>());
                let resp: Response = resp.dyn_into().unwrap();
                resp.array_buffer()
            })
            .and_then(|p: Promise| JsFuture::from(p))
            .and_then(move |buff| {
                console_log!("=== FETCH | buff");
                assert!(buff.is_instance_of::<ArrayBuffer>());
                let typebuf: Uint8Array = Uint8Array::new(&buff);
                let mut body = vec![0; typebuf.length() as usize];
                typebuf.copy_to(&mut body[..]);
                console_log!("=== FETCH | done");
                process2.done(body);
                future::ok(JsValue::null())
            });
        // .and_then(|p: Promise| JsFuture::from(p))
        future_to_promise(future);
        console_log!("=== FETCH | end");
        Ok(Box::new(process))
        // Err(FetchStatus::Canceled(FetchCancelReason::Error))
    }
}
