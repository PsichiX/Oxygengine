use crate::{
    ecs::{
        hierarchy::{hierarchy_system, Hierarchy, HierarchySystemResources},
        life_cycle::{entity_life_cycle_system, EntityChanges, EntityLifeCycleSystemResources},
        pipeline::{PipelineBuilder, PipelineBuilderError, PipelineEngine, PipelineLayer},
        AccessType, Multiverse, System,
    },
    state::{State, StateToken},
    Scalar,
};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

pub trait AppTimer: Send + Sync {
    fn tick(&mut self);
    fn now_since_start(&self) -> Duration;
    fn delta_time(&self) -> Duration;
    fn delta_time_seconds(&self) -> Scalar;
}

pub struct StandardAppTimer {
    timer: Instant,
    delta_time: Duration,
    delta_time_seconds: Scalar,
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
        self.delta_time = d;
        #[cfg(feature = "scalar64")]
        let secs = d.as_secs_f64();
        #[cfg(not(feature = "scalar64"))]
        let secs = d.as_secs_f32();
        self.delta_time_seconds = secs + d.subsec_nanos() as Scalar * 1e-9;
    }

    fn now_since_start(&self) -> Duration {
        self.timer.elapsed()
    }

    fn delta_time(&self) -> Duration {
        self.delta_time
    }

    fn delta_time_seconds(&self) -> Scalar {
        self.delta_time_seconds
    }
}

pub struct AppLifeCycle {
    pub running: bool,
    pub(crate) timer: Box<dyn AppTimer>,
    pub(crate) states_tokens: Vec<StateToken>,
}

impl AppLifeCycle {
    pub fn new(timer: Box<dyn AppTimer>) -> Self {
        Self {
            running: true,
            timer,
            states_tokens: vec![StateToken::new()],
        }
    }

    pub fn now_since_start(&self) -> Duration {
        self.timer.now_since_start()
    }

    pub fn delta_time(&self) -> Duration {
        self.timer.delta_time()
    }

    pub fn delta_time_seconds(&self) -> Scalar {
        self.timer.delta_time_seconds()
    }

    pub fn current_state_token(&self) -> StateToken {
        if let Some(token) = self.states_tokens.last() {
            *token
        } else {
            StateToken::new()
        }
    }
}

pub struct AppRunner {
    pub app: Rc<RefCell<App>>,
}

impl AppRunner {
    pub fn new(app: App) -> Self {
        Self {
            app: Rc::new(RefCell::new(app)),
        }
    }

    pub fn run<BAR, E>(&mut self, mut backend_app_runner: BAR) -> Result<(), E>
    where
        BAR: BackendAppRunner<E>,
    {
        backend_app_runner.run(self.app.clone())
    }
}

pub trait BackendAppRunner<E> {
    fn run(&mut self, app: Rc<RefCell<App>>) -> Result<(), E>;
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

impl BackendAppRunner<()> for SyncAppRunner {
    fn run(&mut self, app: Rc<RefCell<App>>) -> Result<(), ()> {
        while app.borrow().multiverse.is_running() {
            app.borrow_mut().process();
            if let Some(sleep_time) = self.sleep_time {
                std::thread::sleep(sleep_time);
            }
        }
        Ok(())
    }
}

pub struct App {
    pub multiverse: Multiverse,
}

impl App {
    #[inline]
    pub fn build<PB>() -> AppBuilder<PB>
    where
        PB: PipelineBuilder + Default,
    {
        AppBuilder::<PB>::default()
    }

    #[inline]
    pub fn process(&mut self) {
        self.multiverse.process();
    }
}

pub struct AppBuilder<PB>
where
    PB: PipelineBuilder,
{
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    pipeline_builder: PB,
}

impl<PB> Default for AppBuilder<PB>
where
    PB: PipelineBuilder + Default,
{
    fn default() -> Self {
        Self::new(PB::default())
    }
}

impl<PB> AppBuilder<PB>
where
    PB: PipelineBuilder,
{
    #[inline]
    pub fn new(pipeline_builder: PB) -> Self {
        Self {
            resources: Default::default(),
            pipeline_builder,
        }
        .with_resource(EntityChanges::default())
        .with_system_on_layer::<EntityLifeCycleSystemResources>(
            "entity-life-cycle",
            entity_life_cycle_system,
            &[],
            PipelineLayer::Pre,
        )
        .expect("Could not install entity-life-cycle system!")
        .with_resource(Hierarchy::default())
        .with_system_on_layer::<HierarchySystemResources>(
            "hierarchy",
            hierarchy_system,
            &["entity-life-cycle"],
            PipelineLayer::Pre,
        )
        .expect("Could not install hierarchy system!")
    }

    #[inline]
    pub fn pipeline_builder_mut(&mut self) -> &mut PB {
        &mut self.pipeline_builder
    }

    #[inline]
    pub fn install_bundle<ABI, D>(
        &mut self,
        mut installer: ABI,
        data: D,
    ) -> Result<(), PipelineBuilderError>
    where
        ABI: FnMut(&mut AppBuilder<PB>, D) -> Result<(), PipelineBuilderError>,
    {
        installer(self, data)?;
        Ok(())
    }

    #[inline]
    pub fn with_bundle<ABI, D>(
        mut self,
        installer: ABI,
        data: D,
    ) -> Result<Self, PipelineBuilderError>
    where
        ABI: FnMut(&mut AppBuilder<PB>, D) -> Result<(), PipelineBuilderError>,
    {
        self.install_bundle(installer, data)?;
        Ok(self)
    }

    #[inline]
    pub fn install_resource<T>(&mut self, resource: T)
    where
        T: 'static + Send + Sync,
    {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    #[inline]
    pub fn with_resource<T>(mut self, resource: T) -> Self
    where
        T: 'static + Send + Sync,
    {
        self.install_resource(resource);
        self
    }

    #[inline]
    pub fn install_system_on_layer<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<(), PipelineBuilderError> {
        self.pipeline_builder
            .add_system_on_layer::<AT>(name, system, dependencies, layer)?;
        Ok(())
    }

    #[inline]
    pub fn install_system<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
    ) -> Result<(), PipelineBuilderError> {
        self.pipeline_builder
            .add_system::<AT>(name, system, dependencies)?;
        Ok(())
    }

    #[inline]
    pub fn with_system_on_layer<AT: AccessType>(
        mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<Self, PipelineBuilderError> {
        self.install_system_on_layer::<AT>(name, system, dependencies, layer)?;
        Ok(self)
    }

    #[inline]
    pub fn with_system<AT: AccessType>(
        mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
    ) -> Result<Self, PipelineBuilderError> {
        self.install_system::<AT>(name, system, dependencies)?;
        Ok(self)
    }

    #[inline]
    pub fn build<P, S, AT>(self, state: S, app_timer: AT) -> App
    where
        P: PipelineEngine + Send + Sync + Default + 'static,
        S: State + 'static,
        AT: AppTimer + 'static,
    {
        self.build_with_engine(P::default(), state, app_timer)
    }

    pub fn build_with_engine<P, S, AT>(mut self, engine: P, state: S, app_timer: AT) -> App
    where
        P: PipelineEngine + Send + Sync + 'static,
        S: State + 'static,
        AT: AppTimer + 'static,
    {
        self.install_resource(AppLifeCycle::new(Box::new(app_timer)));
        let mut multiverse =
            Multiverse::new(self.pipeline_builder.build_with_engine(engine), state);
        if let Some(universe) = multiverse.default_universe_mut() {
            for (as_type, resource) in self.resources {
                unsafe {
                    universe.insert_resource_raw(as_type, resource);
                }
            }
        }
        App { multiverse }
    }

    #[inline]
    pub fn build_empty<P, AT>(self, app_timer: AT) -> App
    where
        P: PipelineEngine + Send + Sync + Default + 'static,
        AT: AppTimer + 'static,
    {
        self.build::<P, _, _>((), app_timer)
    }

    #[inline]
    pub fn build_empty_with_engine<P, AT>(self, engine: P, app_timer: AT) -> App
    where
        P: PipelineEngine + Send + Sync + 'static,
        AT: AppTimer + 'static,
    {
        self.build_with_engine::<P, _, _>(engine, (), app_timer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{
        pipeline::{engines::jobs::JobsPipelineEngine, ParallelPipelineBuilder},
        Universe,
    };

    #[test]
    fn test_app_builder() {
        struct A;
        struct B;
        struct C;

        fn system(_: &mut Universe) {
            println!("******");
        }

        fn system_a(_: &mut Universe) {
            println!("* Start System A");
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("* Stop System A");
        }

        fn system_b(_: &mut Universe) {
            println!("* Start System B");
            std::thread::sleep(std::time::Duration::from_millis(150));
            println!("* Stop System B");
        }

        fn system_c(_: &mut Universe) {
            println!("* Start System C");
            std::thread::sleep(std::time::Duration::from_millis(50));
            println!("* Stop System C");
        }

        let app = App::build::<ParallelPipelineBuilder>()
            .with_system::<()>("", system, &[])
            .unwrap()
            .with_bundle(
                |builder, _| {
                    builder.install_resource(A);
                    builder.install_resource(B);
                    builder.install_system::<&mut A>("a", system_a, &[])?;
                    builder.install_system::<&mut B>("b", system_b, &[])?;
                    Ok(())
                },
                (),
            )
            .unwrap()
            .with_system::<(&mut C, &A, &B)>("c", system_c, &[])
            .unwrap()
            .with_resource(C)
            .build::<JobsPipelineEngine, _, _>(3, StandardAppTimer::default());

        AppRunner::new(app).run(SyncAppRunner::new()).unwrap();
    }
}
