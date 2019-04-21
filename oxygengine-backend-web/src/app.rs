use core::prelude::*;
use std::{cell::RefCell, rc::Rc, time::Duration};
use wasm_bindgen::{prelude::*, JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn performance() -> web_sys::Performance {
    window()
        .performance()
        .expect("no `window.performance` exists")
}

pub struct WebAppTimer {
    timer: f64,
    delta_time: Duration,
    delta_time_seconds: f64,
}

impl Default for WebAppTimer {
    fn default() -> Self {
        Self {
            timer: performance().now(),
            delta_time: Duration::default(),
            delta_time_seconds: 0.0,
        }
    }
}

impl AppTimer for WebAppTimer {
    fn tick(&mut self) {
        let t = performance().now();
        let d = (t - self.timer) * 0.001;
        self.timer = t;
        let d = Duration::new(d as u64, (d.fract() * 1e9) as u32);
        self.delta_time = d;
        self.delta_time_seconds = d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9;
    }

    fn delta_time(&self) -> Duration {
        self.delta_time
    }

    fn delta_time_seconds(&self) -> f64 {
        self.delta_time_seconds
    }
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
