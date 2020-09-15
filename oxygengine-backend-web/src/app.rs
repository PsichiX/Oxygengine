use core::{
    app::{App, AppLifeCycle, AppTimer, BackendAppRunner},
    ecs::WorldExt,
    Scalar,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
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
        .expect("no `window.performance` exists")
}

pub struct WebAppTimer {
    timer: Scalar,
    delta_time: Duration,
    delta_time_seconds: Scalar,
}

impl Default for WebAppTimer {
    fn default() -> Self {
        Self {
            timer: performance().now() as Scalar * 0.001,
            delta_time: Duration::default(),
            delta_time_seconds: 0.0,
        }
    }
}

impl AppTimer for WebAppTimer {
    fn tick(&mut self) {
        let t = performance().now() as Scalar * 0.001;
        let d = t - self.timer;
        self.timer = t;
        self.delta_time = Duration::new(d as u64, (d.fract() * 1e9) as u32);
        self.delta_time_seconds = d;
    }

    fn delta_time(&self) -> Duration {
        self.delta_time
    }

    fn delta_time_seconds(&self) -> Scalar {
        self.delta_time_seconds
    }
}

#[cfg(feature = "ipc")]
#[derive(Default)]
pub struct WebIpc {
    /// {id, fn(data, origin) -> response?}
    callbacks: std::collections::HashMap<String, Box<dyn FnMut(JsValue, &str) -> Option<JsValue>>>,
}

#[cfg(feature = "ipc")]
impl WebIpc {
    pub fn register(
        &mut self,
        id: String,
        callback: Box<dyn FnMut(JsValue, &str) -> Option<JsValue>>,
    ) {
        self.callbacks.insert(id, callback);
    }

    pub fn unregister(&mut self, id: &str) {
        self.callbacks.remove(id);
    }
}

#[derive(Default)]
pub struct WebAppRunner;

impl BackendAppRunner<'static, 'static, JsValue> for WebAppRunner {
    fn run(&mut self, app: Rc<RefCell<App<'static, 'static>>>) -> Result<(), JsValue> {
        #[cfg(feature = "ipc")]
        let app2 = Rc::clone(&app);
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !app.borrow().world().read_resource::<AppLifeCycle>().running {
                drop(f.borrow_mut().take());
                return;
            }
            app.borrow_mut().process();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
        #[cfg(feature = "ipc")]
        {
            let f = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                for cb in app2
                    .borrow()
                    .world()
                    .write_resource::<WebIpc>()
                    .callbacks
                    .values_mut()
                {
                    if let Some(response) = (cb)(event.data(), &event.origin()) {
                        if let Some(source) = event.source() {
                            source
                                .dyn_ref::<web_sys::Window>()
                                .expect("Could not convert sender to `Window`")
                                .post_message(&response, &event.origin())
                                .expect("Could not post message back to the sender");
                        }
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>);
            window()
                .add_event_listener_with_callback("message", f.as_ref().unchecked_ref())
                .expect("Could not perform `add_event_listener`");
            f.forget();
        }
        Ok(())
    }
}
