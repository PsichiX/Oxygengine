use crate::{
    hierarchy::Parent,
    state::{EmptyState, State, StateChange},
};
use specs::{Component, Dispatcher, DispatcherBuilder, ReaderId, RunNow, System, World};
use specs_hierarchy::{Hierarchy, HierarchyEvent, HierarchySystem};
use std::{
    cell::RefCell,
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
        self.delta_time_seconds = d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9;
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

    pub fn run<BAR, E>(&mut self) -> Result<(), E>
    where
        BAR: BackendAppRunner<'a, 'b, E>,
    {
        BAR::run(self.app.clone())
    }
}

pub trait BackendAppRunner<'a, 'b, E> {
    fn run(app: Rc<RefCell<App<'a, 'b>>>) -> Result<(), E> {
        while app.borrow().world().read_resource::<AppLifeCycle>().running {
            app.borrow_mut().process();
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncAppRunner;

impl<'a, 'b> BackendAppRunner<'a, 'b, ()> for SyncAppRunner {}

pub struct App<'a, 'b> {
    world: World,
    states: Vec<Box<dyn State>>,
    dispatcher: Dispatcher<'a, 'b>,
    setup: bool,
    hierarchy_change_event: ReaderId<HierarchyEvent>,
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
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        {
            let entities = {
                let hierarchy = &self.world.read_resource::<Hierarchy<Parent>>();
                hierarchy
                    .changed()
                    .read(&mut self.hierarchy_change_event)
                    .filter_map(|event| {
                        if let HierarchyEvent::Removed(entity) = event {
                            Some(*entity)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };
            for entity in entities {
                drop(self.world.delete_entity(entity));
            }
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
            let lifecycle: &mut AppLifeCycle = &mut self.world.write_resource::<AppLifeCycle>();
            lifecycle.timer.tick();
        }
    }
}

#[derive(Default)]
pub struct AppBuilder<'a, 'b> {
    world: World,
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> AppBuilder<'a, 'b> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
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
        self.dispatcher_builder
            .add(HierarchySystem::<Parent>::new(), "hierarchy", &[]);
        self.world
            .add_resource(AppLifeCycle::new(Box::new(app_timer)));
        let mut dispatcher = self.dispatcher_builder.build();
        dispatcher.setup(&mut self.world.res);
        let hierarchy_change_event = {
            let hierarchy = &mut self.world.write_resource::<Hierarchy<Parent>>();
            hierarchy.track()
        };
        App {
            world: self.world,
            states: vec![Box::new(state)],
            dispatcher,
            setup: true,
            hierarchy_change_event,
        }
    }

    pub fn build_empty<AT>(self, app_timer: AT) -> App<'a, 'b>
    where
        AT: AppTimer + 'static,
    {
        self.build(EmptyState, app_timer)
    }
}
