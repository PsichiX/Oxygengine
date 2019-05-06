use crate::{
    hierarchy::{HierarchyChangeRes, Name, Parent, Tag},
    state::{State, StateChange},
};
use specs::{
    world::EntitiesRes, Component, Dispatcher, DispatcherBuilder, Entity, Join, RunNow, System,
    World,
};
use specs_hierarchy::HierarchySystem;
use std::{
    cell::RefCell,
    collections::HashSet,
    rc::Rc,
    time::{Duration, Instant},
};

pub trait AppTimer: Send + Sync {
    fn tick(&mut self);
    fn delta_time(&self) -> Duration;
    fn delta_time_seconds(&self) -> f64;
}

pub struct StandardAppTimer {
    timer: Instant,
    delta_time: Duration,
    delta_time_seconds: f64,
}

impl Default for StandardAppTimer {
    fn default() -> Self {
        Self {
            timer: Instant::now(),
            delta_time: Duration::default(),
            delta_time_seconds: 0.0,
        }
    }
}

impl AppTimer for StandardAppTimer {
    fn tick(&mut self) {
        let d = self.timer.elapsed();
        self.timer = Instant::now();
        self.delta_time = d;
        self.delta_time_seconds = d.as_secs() as f64 + f64::from(d.subsec_nanos()) * 1e-9;
    }

    fn delta_time(&self) -> Duration {
        self.delta_time
    }

    fn delta_time_seconds(&self) -> f64 {
        self.delta_time_seconds
    }
}

pub struct AppLifeCycle {
    pub running: bool,
    pub(crate) timer: Box<dyn AppTimer>,
}

impl AppLifeCycle {
    pub fn new(timer: Box<dyn AppTimer>) -> Self {
        Self {
            running: true,
            timer,
        }
    }

    pub fn delta_time(&self) -> Duration {
        self.timer.delta_time()
    }

    pub fn delta_time_seconds(&self) -> f64 {
        self.timer.delta_time_seconds()
    }
}

pub struct AppRunner<'a, 'b> {
    pub app: Rc<RefCell<App<'a, 'b>>>,
}

impl<'a, 'b> AppRunner<'a, 'b> {
    pub fn new(app: App<'a, 'b>) -> Self {
        Self {
            app: Rc::new(RefCell::new(app)),
        }
    }

    pub fn run<BAR, E>(&mut self, mut backend_app_runner: BAR) -> Result<(), E>
    where
        BAR: BackendAppRunner<'a, 'b, E>,
    {
        backend_app_runner.run(self.app.clone())
    }
}

pub trait BackendAppRunner<'a, 'b, E> {
    fn run(&mut self, app: Rc<RefCell<App<'a, 'b>>>) -> Result<(), E>;
}

#[derive(Default)]
pub struct SyncAppRunner {
    pub sleep_time: Option<Duration>,
}

impl SyncAppRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sleep_time(value: Duration) -> Self {
        Self {
            sleep_time: Some(value),
        }
    }
}

impl<'a, 'b> BackendAppRunner<'a, 'b, ()> for SyncAppRunner {
    fn run(&mut self, app: Rc<RefCell<App<'a, 'b>>>) -> Result<(), ()> {
        while app.borrow().world().read_resource::<AppLifeCycle>().running {
            app.borrow_mut().process();
            if let Some(sleep_time) = self.sleep_time {
                std::thread::sleep(sleep_time);
            }
        }
        Ok(())
    }
}

pub struct App<'a, 'b> {
    world: World,
    states: Vec<Box<dyn State>>,
    dispatcher: Dispatcher<'a, 'b>,
    setup: bool,
}

impl<'a, 'b> App<'a, 'b> {
    #[inline]
    pub fn build() -> AppBuilder<'a, 'b> {
        AppBuilder::default()
    }

    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[inline]
    pub fn process(&mut self) {
        if self.states.is_empty() {
            self.world.write_resource::<AppLifeCycle>().running = false;
            return;
        }
        if self.setup {
            self.states.last_mut().unwrap().on_enter(&mut self.world);
            self.setup = false;
        }
        let count = self.states.len() - 1;
        for state in self.states.iter_mut().take(count) {
            state.on_process_background(&mut self.world);
        }
        let change = self.states.last_mut().unwrap().on_process(&mut self.world);
        self.dispatcher.dispatch(&self.world.res);
        self.world.maintain();
        {
            let mut changes = self.world.write_resource::<HierarchyChangeRes>();
            changes.added.clear();
            changes.removed.clear();
            let entities = self
                .world
                .read_resource::<EntitiesRes>()
                .join()
                .collect::<HashSet<_>>();
            let ptr = &mut changes.removed as *mut Vec<Entity>;
            for entity in changes.entities.difference(&entities) {
                unsafe {
                    (&mut *ptr).push(*entity);
                }
            }
            let ptr = &mut changes.added as *mut Vec<Entity>;
            for entity in entities.difference(&changes.entities) {
                unsafe {
                    (&mut *ptr).push(*entity);
                }
            }
            changes.entities = entities;
        }
        match change {
            StateChange::Push(mut state) => {
                self.states.last_mut().unwrap().on_pause(&mut self.world);
                state.on_enter(&mut self.world);
                self.states.push(state);
            }
            StateChange::Pop => {
                self.states.pop().unwrap().on_exit(&mut self.world);
                if let Some(state) = self.states.last_mut() {
                    state.on_resume(&mut self.world);
                }
            }
            StateChange::Swap(mut state) => {
                self.states.pop().unwrap().on_exit(&mut self.world);
                state.on_enter(&mut self.world);
                self.states.push(state);
            }
            StateChange::Quit => {
                while let Some(mut state) = self.states.pop() {
                    state.on_exit(&mut self.world);
                }
            }
            _ => {}
        }
        {
            let lifecycle = &mut self.world.write_resource::<AppLifeCycle>();
            lifecycle.timer.tick();
        }
    }
}

pub struct AppBuilder<'a, 'b> {
    world: World,
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for AppBuilder<'a, 'b> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'b> AppBuilder<'a, 'b> {
    #[inline]
    pub fn new() -> Self {
        let mut result = Self {
            world: Default::default(),
            dispatcher_builder: Default::default(),
        };
        result
            .dispatcher_builder
            .add(HierarchySystem::<Parent>::new(), "hierarchy", &[]);
        result
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[inline]
    pub fn with_bundle<ABI, D>(mut self, mut installer: ABI, data: D) -> Self
    where
        ABI: FnMut(&mut AppBuilder<'a, 'b>, D),
    {
        installer(&mut self, data);
        self
    }

    #[inline]
    pub fn with_system<T>(mut self, system: T, name: &str, deps: &[&str]) -> Self
    where
        T: for<'c> System<'c> + Send + 'a,
    {
        self.dispatcher_builder.add(system, name, deps);
        self
    }

    #[inline]
    pub fn with_thread_local_system<T>(mut self, system: T) -> Self
    where
        T: for<'c> RunNow<'c> + 'b,
    {
        self.dispatcher_builder.add_thread_local(system);
        self
    }

    #[inline]
    pub fn with_barrier(mut self) -> Self {
        self.dispatcher_builder.add_barrier();
        self
    }

    #[inline]
    pub fn with_resource<T>(mut self, resource: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.world.add_resource(resource);
        self
    }

    #[inline]
    pub fn with_component<T: Component>(mut self) -> Self
    where
        T::Storage: Default,
    {
        self.world.register::<T>();
        self
    }

    #[inline]
    pub fn install_bundle<ABI, D>(&mut self, mut installer: ABI, data: D)
    where
        ABI: FnMut(&mut AppBuilder<'a, 'b>, D),
    {
        installer(self, data);
    }

    #[inline]
    pub fn install_system<T>(&mut self, system: T, name: &str, deps: &[&str])
    where
        T: for<'c> System<'c> + Send + 'a,
    {
        self.dispatcher_builder.add(system, name, deps);
    }

    #[inline]
    pub fn install_thread_local_system<T>(&mut self, system: T)
    where
        T: for<'c> RunNow<'c> + 'b,
    {
        self.dispatcher_builder.add_thread_local(system);
    }

    #[inline]
    pub fn install_barrier(&mut self) {
        self.dispatcher_builder.add_barrier();
    }

    #[inline]
    pub fn install_resource<T>(&mut self, resource: T)
    where
        T: Send + Sync + 'static,
    {
        self.world.add_resource(resource);
    }

    #[inline]
    pub fn install_component<T: Component>(&mut self)
    where
        T::Storage: Default,
    {
        self.world.register::<T>();
    }

    pub fn build<S, AT>(mut self, state: S, app_timer: AT) -> App<'a, 'b>
    where
        S: State + 'static,
        AT: AppTimer + 'static,
    {
        self.world
            .add_resource(AppLifeCycle::new(Box::new(app_timer)));
        self.world.add_resource(HierarchyChangeRes::default());
        self.world.register::<Parent>();
        self.world.register::<Name>();
        self.world.register::<Tag>();
        let mut dispatcher = self.dispatcher_builder.build();
        dispatcher.setup(&mut self.world.res);
        App {
            world: self.world,
            states: vec![Box::new(state)],
            dispatcher,
            setup: true,
        }
    }

    pub fn build_empty<AT>(self, app_timer: AT) -> App<'a, 'b>
    where
        AT: AppTimer + 'static,
    {
        self.build((), app_timer)
    }
}
