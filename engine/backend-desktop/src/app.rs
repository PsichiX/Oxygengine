use crate::resource::DesktopAppEvents;
use core::app::{App, AppLifeCycle, AppParams, BackendAppRunner};
use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Fullscreen, Window, WindowBuilder},
    ContextBuilder, ContextWrapper, PossiblyCurrent,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    env::{current_exe, set_current_dir, var},
    rc::Rc,
    sync::Arc,
};

pub type DesktopContextWrapper = ContextWrapper<PossiblyCurrent, Window>;

pub fn desktop_app_params() -> AppParams {
    let mut result = HashMap::default();
    let mut key = None;
    for arg in std::env::args() {
        if let Some(value) = arg.strip_prefix("--") {
            key = Some(value.to_owned());
            result.insert(value.to_owned(), Default::default());
        } else if let Some(value) = arg.strip_prefix('-') {
            key = Some(value.to_owned());
            result.insert(value.to_owned(), Default::default());
        } else if let Some(value) = arg.strip_prefix('/') {
            key = Some(value.to_owned());
            result.insert(value.to_owned(), Default::default());
        } else if let Some(key) = key.take() {
            result.insert(key, arg.to_owned());
        }
    }
    AppParams::new(result)
}

pub struct DesktopAppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub depth: bool,
    pub stencil: bool,
}

impl Default for DesktopAppConfig {
    fn default() -> Self {
        Self {
            title: "Oxygengine Game".to_owned(),
            width: 1024,
            height: 576,
            fullscreen: false,
            vsync: false,
            depth: false,
            stencil: false,
        }
    }
}

pub struct DesktopAppRunner {
    event_loop: EventLoop<()>,
    context_wrapper: Arc<DesktopContextWrapper>,
}

impl DesktopAppRunner {
    pub fn new(config: DesktopAppConfig) -> Self {
        if let Ok(path) = var("OXY_ROOT_PATH") {
            let _ = set_current_dir(path);
        } else if let Ok(mut dir) = current_exe() {
            dir.pop();
            let _ = set_current_dir(dir);
        }
        let DesktopAppConfig {
            title,
            width,
            height,
            fullscreen,
            vsync,
            depth,
            stencil,
        } = config;
        let fullscreen = if fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        };
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_title(title.as_str())
            .with_inner_size(LogicalSize::new(width, height))
            .with_fullscreen(fullscreen);
        let context_wrapper = Arc::new(unsafe {
            let mut builder = ContextBuilder::new()
                .with_vsync(vsync)
                .with_double_buffer(Some(true))
                .with_hardware_acceleration(Some(true));
            if depth {
                builder = builder.with_depth_buffer(24);
            }
            if stencil {
                builder = builder.with_stencil_buffer(8);
            }
            builder
                .build_windowed(window_builder, &event_loop)
                .expect("Could not build windowed context wrapper!")
                .make_current()
                .expect("Could not make windowed context wrapper a current one!")
        });
        Self {
            event_loop,
            context_wrapper,
        }
    }

    pub fn context_wrapper(&self) -> Arc<DesktopContextWrapper> {
        self.context_wrapper.clone()
    }
}

impl BackendAppRunner<()> for DesktopAppRunner {
    fn run(&mut self, app: Rc<RefCell<App>>) -> Result<(), ()> {
        let mut running = true;
        while running {
            self.event_loop.run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::MainEventsCleared => {
                        let mut app = app.borrow_mut();
                        app.process();
                        app.multiverse
                            .default_universe_mut()
                            .unwrap()
                            .expect_resource_mut::<DesktopAppEvents>()
                            .clear();
                        let _ = self.context_wrapper.swap_buffers();
                        if !app.multiverse.is_running() {
                            running = false;
                        }
                        *control_flow = ControlFlow::Exit;
                    }
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            self.context_wrapper.resize(physical_size);
                            app.borrow_mut()
                                .multiverse
                                .default_universe_mut()
                                .unwrap()
                                .expect_resource_mut::<DesktopAppEvents>()
                                .push(event.to_static().unwrap());
                        }
                        WindowEvent::CloseRequested => {
                            for universe in app.borrow_mut().multiverse.universes_mut() {
                                universe.expect_resource_mut::<AppLifeCycle>().running = false;
                            }
                        }
                        event => {
                            if let Some(event) = event.to_static() {
                                app.borrow_mut()
                                    .multiverse
                                    .default_universe_mut()
                                    .unwrap()
                                    .expect_resource_mut::<DesktopAppEvents>()
                                    .push(event);
                            }
                        }
                    },
                    _ => {}
                }
            });
        }
        Ok(())
    }
}
