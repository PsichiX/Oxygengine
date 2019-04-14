use core::prelude::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub struct WebAppRunner;

impl BackendAppRunner<'static, 'static, JsValue> for WebAppRunner {
    fn run(app: Rc<RefCell<App<'static, 'static>>>) -> Result<(), JsValue> {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !app.borrow().world().read_resource::<AppLifeCycle>().running {
                drop(f.borrow_mut().take());
                return;
            }
            app.borrow_mut().process();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
        Ok(())
    }
}
