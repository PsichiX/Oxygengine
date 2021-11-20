#![cfg(feature = "web")]

use crate::platform::{HaPlatformInterface, HaPlatformInterfaceProcessResult};
use glow::*;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, *};
use web_sys::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebPlatformInterfaceError {
    CanvasNotFound(String),
}

impl From<WebPlatformInterfaceError> for JsValue {
    fn from(error: WebPlatformInterfaceError) -> Self {
        JsValue::from_str(&format!("{:?}", error))
    }
}

fn get_canvas_by_id(id: &str) -> Result<HtmlCanvasElement, WebPlatformInterfaceError> {
    let canvas = web_sys::window()
        .expect("Could not access `window`")
        .document()
        .expect("Could not access `window.document`")
        .get_element_by_id(id);
    if let Some(canvas) = canvas {
        Ok(canvas
            .dyn_into::<HtmlCanvasElement>()
            .expect("DOM element is not HtmlCanvasElement"))
    } else {
        Err(WebPlatformInterfaceError::CanvasNotFound(id.to_owned()))
    }
}

#[cfg(target_arch = "wasm32")]
fn get_webgl2_context(
    canvas: &HtmlCanvasElement,
    options: &WebContextOptions,
) -> Option<WebGl2RenderingContext> {
    let options = options.as_js_value();
    if let Ok(Some(context)) = canvas.get_context_with_context_options("webgl2", &options) {
        Some(
            context
                .dyn_into::<WebGl2RenderingContext>()
                .expect("DOM element is not WebGl2RenderingContext"),
        )
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn get_glow_context(canvas: &HtmlCanvasElement, options: &WebContextOptions) -> Option<Context> {
    get_webgl2_context(canvas, options).map(|context| Context::from_webgl2_context(context))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_glow_context(_: &HtmlCanvasElement, _: &WebContextOptions) -> Option<Context> {
    unreachable!()
}

fn listen_for_events(
    canvas: &HtmlCanvasElement,
    context_lost: Rc<RefCell<bool>>,
    context_restored: Rc<RefCell<bool>>,
) {
    {
        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            *context_lost.borrow_mut() = true;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("webglcontextlost", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
    {
        let closure = Closure::wrap(Box::new(move |_: Event| {
            *context_restored.borrow_mut() = true;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback(
                "webglcontextrestored",
                closure.as_ref().unchecked_ref(),
            )
            .unwrap();
        closure.forget();
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct WebContextOptions {
    pub alpha: bool,
    pub depth: bool,
    pub stencil: bool,
}

impl WebContextOptions {
    #[cfg(target_arch = "wasm32")]
    fn as_js_value(&self) -> JsValue {
        let mut options = HashMap::new();
        options.insert("alpha", self.alpha);
        options.insert("depth", self.depth);
        options.insert("stencil", self.stencil);
        JsValue::from_serde(&options).expect("Could not construct WebGL 2 context options map")
    }
}

#[derive(Debug)]
pub struct WebPlatformInterface {
    canvas: HtmlCanvasElement,
    options: WebContextOptions,
    context: Option<Context>,
    cached_screen_size: (usize, usize),
    context_lost: Rc<RefCell<bool>>,
    context_restored: Rc<RefCell<bool>>,
}

unsafe impl Send for WebPlatformInterface {}
unsafe impl Sync for WebPlatformInterface {}

impl WebPlatformInterface {
    pub fn with_canvas_id(
        canvas_id: &str,
        options: WebContextOptions,
    ) -> Result<Self, WebPlatformInterfaceError> {
        Ok(Self::with_canvas(get_canvas_by_id(canvas_id)?, options))
    }

    pub fn with_canvas(canvas: HtmlCanvasElement, options: WebContextOptions) -> Self {
        let context = get_glow_context(&canvas, &options);
        let context_lost = Rc::new(RefCell::new(false));
        let context_restored = Rc::new(RefCell::new(false));
        listen_for_events(
            &canvas,
            Rc::clone(&context_lost),
            Rc::clone(&context_restored),
        );
        Self {
            canvas,
            options,
            context,
            cached_screen_size: (0, 0),
            context_lost,
            context_restored,
        }
    }
}

impl HaPlatformInterface for WebPlatformInterface {
    fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    fn screen_size(&self) -> (usize, usize) {
        self.cached_screen_size
    }

    fn maintain<'a>(&'a mut self) -> HaPlatformInterfaceProcessResult<'a> {
        let mut result = HaPlatformInterfaceProcessResult::default();
        let context_lost = { *self.context_lost.borrow() };
        let context_restored = { *self.context_restored.borrow() || self.context.is_none() };
        if context_lost {
            result.context_lost = std::mem::take(&mut self.context);
            self.cached_screen_size = (0, 0);
        }
        if context_restored {
            self.context = get_glow_context(&self.canvas, &self.options);
            result.context_acquired = self.context.as_ref();
        }
        if self.context.is_none() {
            return result;
        }
        let cw = self.canvas.client_width().max(1) as _;
        let w = self.canvas.width();
        if cw != w {
            self.canvas.set_width(cw);
        }
        let ch = self.canvas.client_height().max(1) as _;
        let h = self.canvas.height();
        if ch != h {
            self.canvas.set_height(ch);
        }
        let cw = cw as _;
        let ch = ch as _;
        if cw != self.cached_screen_size.0 || ch != self.cached_screen_size.1 {
            self.cached_screen_size.0 = cw;
            self.cached_screen_size.1 = ch;
            result.screen_resized = Some((cw, ch));
        }
        result
    }
}
