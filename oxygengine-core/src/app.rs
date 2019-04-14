use specs::{Component, Dispatcher, DispatcherBuilder, RunNow, System, World};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct AppLifeCycle {
    pub running: bool,
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
        self.app
            .borrow_mut()
            .world_mut()
            .add_resource(AppLifeCycle { running: true });
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
    dispatcher: Dispatcher<'a, 'b>,
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
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
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

    pub fn build(mut self) -> App<'a, 'b> {
        let mut dispatcher = self.dispatcher_builder.build();
        dispatcher.setup(&mut self.world.res);
        App {
            world: self.world,
            dispatcher,
        }
    }
}
