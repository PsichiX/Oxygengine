use core::{
    app::{App, AppParams, AppTimer, BackendAppRunner},
    Scalar,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use url::Url;
use wasm_bindgen::{prelude::*, JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Could not perform `requestAnimationFrame`");
}

fn performance() -> web_sys::Performance {
    window()
        .performance()
        .expect("`window.performance` does not exists")
}

pub fn web_app_params() -> AppParams {
    let url = window()
        .document()
        .expect("`window.document` does not exists")
        .location()
        .expect("`window.document.location` does not exists")
        .href()
        .expect("`window.document.location.href` does not exists");
    AppParams::new(
        Url::parse(&url)
            .expect("Could not parse application URL")
            .query_pairs()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

pub struct WebAppTimer {
    timer: Scalar,
    last_timer: Scalar,
    time: Duration,
    time_seconds: Scalar,
    delta_time: Duration,
    delta_time_seconds: Scalar,
    ticks: usize,
}

impl Default for WebAppTimer {
    fn default() -> Self {
        let current_time = performance().now() as Scalar * 0.001;
        Self {
            timer: current_time,
            last_timer: current_time,
            time: Duration::default(),
            time_seconds: 0.0,
            delta_time: Duration::default(),
            delta_time_seconds: 0.0,
            ticks: 0,
        }
    }
}

impl AppTimer for WebAppTimer {
    fn tick(&mut self) {
        let current_time = performance().now() as Scalar * 0.001;
        self.delta_time_seconds = current_time - self.last_timer;
        self.delta_time = Duration::new(
            self.delta_time_seconds as u64,
            (self.delta_time_seconds.fract() * 1e9) as u32,
        );
        self.time_seconds = current_time - self.timer;
        self.time = Duration::new(
            self.time_seconds as u64,
            (self.time_seconds.fract() * 1e9) as u32,
        );
        self.last_timer = current_time;
        self.ticks = self.ticks.wrapping_add(1);
    }

    fn time(&self) -> Duration {
        self.time
    }

    fn time_seconds(&self) -> Scalar {
        self.time_seconds
    }

    fn delta_time(&self) -> Duration {
        self.delta_time
    }

    fn delta_time_seconds(&self) -> Scalar {
        self.delta_time_seconds
    }

    fn ticks(&self) -> usize {
        self.ticks
    }
}

#[derive(Default)]
pub struct WebAppRunner;

impl BackendAppRunner<JsValue> for WebAppRunner {
    fn run(&mut self, app: Rc<RefCell<App>>) -> Result<(), JsValue> {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !app.borrow().multiverse.is_running() {
                drop(f.borrow_mut().take());
                return;
            }
            app.borrow_mut().process();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
        Ok(())
    }
}
