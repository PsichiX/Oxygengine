use core::fetch::{FetchEngine, FetchProcess, FetchProcessReader, FetchStatus};
use futures::{future, Future};
use js_sys::*;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::*;

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
    fn fetch(&mut self, path: &str) -> Result<Box<dyn FetchProcessReader>, FetchStatus> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let full_path = format!("{}/{}", self.root_path, path);
        let request = Request::new_with_str_and_init(&full_path, &opts).unwrap();
        let request_promise = window().fetch_with_request(&request);
        let process = FetchProcess::new_start();
        let mut process2 = process.clone();
        let future = JsFuture::from(request_promise)
            .and_then(|resp| {
                assert!(resp.is_instance_of::<Response>());
                let resp: Response = resp.dyn_into().unwrap();
                resp.array_buffer()
            })
            .and_then(JsFuture::from)
            .and_then(move |buff| {
                assert!(buff.is_instance_of::<ArrayBuffer>());
                let typebuf: Uint8Array = Uint8Array::new(&buff);
                let mut body = vec![0; typebuf.length() as usize];
                typebuf.copy_to(&mut body[..]);
                process2.done(body);
                future::ok(JsValue::null())
            });
        // TODO: fail process on error catch.
        drop(future_to_promise(future));
        Ok(Box::new(process))
    }
}
