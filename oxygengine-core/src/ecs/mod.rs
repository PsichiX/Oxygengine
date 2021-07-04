pub mod components;
pub mod hierarchy;
pub mod life_cycle;
pub mod pipeline;

use crate::{
    app::AppLifeCycle,
    ecs::{
        components::NonPersistent,
        pipeline::{PipelineEngine, PipelineID},
    },
    state::{State, StateChange, StateToken},
};
pub use hecs::*;
use std::{
    any::{type_name, Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet, VecDeque},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};
use typid::ID;

pub type System = fn(&mut Universe);
pub type UniverseID = ID<Universe>;
pub type Resource = dyn Any + Send + Sync;

pub struct WorldRef;
pub struct WorldMut;
pub struct Comp<T>(PhantomData<T>);

#[derive(Default)]
pub struct Multiverse {
    pub parallel: bool,
    universes: HashMap<UniverseID, Universe>,
    pipelines: HashMap<PipelineID, Box<dyn PipelineEngine + Send + Sync>>,
    bindings: HashMap<UniverseID, PipelineID>,
    default_universe: Option<UniverseID>,
}

impl Multiverse {
    pub fn new<T, S>(pipeline: T, state: S) -> Self
    where
        T: PipelineEngine + 'static + Send + Sync,
        S: State + 'static,
    {
        let mut result = Self::default();
        let universe = result.create_universe(state);
        let pipeline = result.insert_pipeline(pipeline);
        result.bind(universe, pipeline);
        result.set_default_universe_id(Some(universe));
        result
    }

    pub fn with_parallel(mut self, mode: bool) -> Self {
        self.parallel = mode;
        self
    }

    pub fn create_universe<S>(&mut self, state: S) -> UniverseID
    where
        S: State + 'static,
    {
        let id = UniverseID::new();
        self.universes.insert(id, Universe::new(state));
        id
    }

    pub fn delete_universe(&mut self, id: UniverseID) -> Option<Universe> {
        if let Some(uni) = self.default_universe {
            if uni == id {
                self.default_universe = None;
            }
        }
        self.bindings.remove(&id);
        self.universes.remove(&id)
    }

    pub fn default_universe_id(&self) -> Option<UniverseID> {
        self.default_universe
    }

    pub fn set_default_universe_id(&mut self, id: Option<UniverseID>) {
        self.default_universe = id;
    }

    pub fn default_universe(&self) -> Option<&Universe> {
        if let Some(id) = self.default_universe {
            self.universe(id)
        } else {
            None
        }
    }

    pub fn default_universe_mut(&mut self) -> Option<&mut Universe> {
        if let Some(id) = self.default_universe {
            self.universe_mut(id)
        } else {
            None
        }
    }

    pub fn universe(&self, id: UniverseID) -> Option<&Universe> {
        self.universes.get(&id)
    }

    pub fn universe_mut(&mut self, id: UniverseID) -> Option<&mut Universe> {
        self.universes.get_mut(&id)
    }

    pub fn universe_ids(&self) -> impl Iterator<Item = UniverseID> + '_ {
        self.universes.keys().cloned()
    }

    pub fn universes(&self) -> impl Iterator<Item = &Universe> {
        self.universes.values()
    }

    pub fn universes_mut(&mut self) -> impl Iterator<Item = &mut Universe> {
        self.universes.values_mut()
    }

    pub fn universes_with_ids(&self) -> impl Iterator<Item = (UniverseID, &Universe)> {
        self.universes.iter().map(|(id, u)| (*id, u))
    }

    pub fn universes_with_ids_mut(&mut self) -> impl Iterator<Item = (UniverseID, &mut Universe)> {
        self.universes.iter_mut().map(|(id, u)| (*id, u))
    }

    pub fn insert_pipeline<T>(&mut self, pipeline: T) -> PipelineID
    where
        T: PipelineEngine + 'static + Send + Sync,
    {
        let id = PipelineID::new();
        self.pipelines.insert(id, Box::new(pipeline));
        id
    }

    pub fn remove_pipeline(&mut self, id: PipelineID) {
        self.bindings.retain(|_, p| p != &id);
        self.pipelines.remove(&id);
    }

    pub fn pipeline_ids(&self) -> impl Iterator<Item = PipelineID> + '_ {
        self.pipelines.keys().cloned()
    }

    pub fn bind(&mut self, universe: UniverseID, pipeline: PipelineID) {
        self.bindings.insert(universe, pipeline);
    }

    pub fn unbind(&mut self, universe: UniverseID) {
        self.bindings.remove(&universe);
    }

    pub fn unbind_all(&mut self) {
        self.bindings.clear();
    }

    pub fn is_running(&self) -> bool {
        self.bindings.keys().any(|id| {
            self.universes
                .get(id)
                .map(|u| u.is_running())
                .unwrap_or_default()
        })
    }

    pub fn process(&mut self) {
        #[cfg(not(feature = "parallel"))]
        {
            for (universe, pipeline) in &self.bindings {
                if let (Some(universe), Some(pipeline)) = (
                    self.universes.get_mut(universe),
                    self.pipelines.get(pipeline),
                ) {
                    pipeline.run(universe);
                }
            }
            for universe in self.universes.values_mut() {
                universe.commands().run(universe);
                universe.maintain();
            }
        }
        #[cfg(feature = "parallel")]
        {
            if self.parallel && self.bindings.len() > 1 {
                use rayon::prelude::*;
                let bindings = self
                    .bindings
                    .iter()
                    .map(|(u, p)| (*u, *p))
                    .collect::<Vec<_>>();
                bindings.into_par_iter().for_each(|(universe, pipeline)| {
                    if let (Some(universe), Some(pipeline)) =
                        (self.universes.get(&universe), self.pipelines.get(&pipeline))
                    {
                        #[allow(mutable_transmutes)]
                        #[allow(clippy::transmute_ptr_to_ptr)]
                        pipeline.run(unsafe { std::mem::transmute(universe) });
                    }
                });
                self.universes
                    .par_iter_mut()
                    .for_each(|(_, universe)| universe.maintain());
            } else {
                for (universe, pipeline) in &self.bindings {
                    if let (Some(universe), Some(pipeline)) = (
                        self.universes.get_mut(universe),
                        self.pipelines.get(pipeline),
                    ) {
                        pipeline.run(universe);
                    }
                }
                for universe in self.universes.values_mut() {
                    universe.commands().run(universe);
                    universe.maintain();
                }
            }
        }
    }
}

pub trait UniverseCommand: Send + Sync {
    fn run(&mut self, universe: &mut Universe);
}

struct ClosureUniverseCommand(Box<dyn FnMut(&mut Universe) + Send + Sync>);

impl UniverseCommand for ClosureUniverseCommand {
    fn run(&mut self, universe: &mut Universe) {
        (self.0)(universe);
    }
}

#[derive(Default, Clone)]
pub struct UniverseCommands {
    queue: Arc<RwLock<VecDeque<Box<dyn UniverseCommand>>>>,
}

impl UniverseCommands {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::with_capacity(size))),
        }
    }

    pub fn schedule<T>(&mut self, command: T) -> bool
    where
        T: UniverseCommand + 'static,
    {
        if let Ok(mut queue) = self.queue.try_write() {
            queue.push_back(Box::new(command));
            true
        } else {
            false
        }
    }

    pub fn schedule_fn<F>(&mut self, f: F) -> bool
    where
        F: FnMut(&mut Universe) + Send + Sync + 'static,
    {
        self.schedule(ClosureUniverseCommand(Box::new(f)))
    }

    pub fn run(&mut self, universe: &mut Universe) {
        if let Ok(mut queue) = self.queue.try_write() {
            while let Some(mut command) = queue.pop_front() {
                command.run(universe);
            }
            queue.clear();
        }
    }
}

pub struct Universe {
    resources: HashMap<TypeId, RefCell<Box<Resource>>>,
    states: Vec<Box<dyn State>>,
    startup: bool,
    world: RefCell<World>,
    commands: UniverseCommands,
}

unsafe impl Send for Universe {}
unsafe impl Sync for Universe {}

impl Default for Universe {
    fn default() -> Self {
        Self {
            resources: Default::default(),
            states: vec![],
            startup: true,
            world: Default::default(),
            commands: UniverseCommands::with_capacity(1024),
        }
    }
}

impl Universe {
    pub fn new<S>(state: S) -> Self
    where
        S: State + 'static,
    {
        Self {
            resources: Default::default(),
            states: vec![Box::new(state)],
            startup: true,
            world: Default::default(),
            commands: UniverseCommands::with_capacity(1024),
        }
    }

    pub fn world(&self) -> Ref<World> {
        self.world.borrow()
    }

    pub fn world_mut(&self) -> RefMut<World> {
        self.world.borrow_mut()
    }

    pub fn try_world(&self) -> Option<Ref<World>> {
        match self.world.try_borrow() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn try_world_mut(&self) -> Option<RefMut<World>> {
        match self.world.try_borrow_mut() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn commands(&self) -> UniverseCommands {
        self.commands.clone()
    }

    pub fn insert_resource<T>(&mut self, resource: T)
    where
        T: 'static + Send + Sync,
    {
        self.resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    /// # Safety
    /// This function assume that `as_type` matches exactly the type of `resource`, you can call it
    /// for example if you want to move already prepared resources from another place to this
    /// universe (in this case we can be sure type IDs matches the types of resources).
    pub unsafe fn insert_resource_raw(&mut self, as_type: TypeId, resource: Box<Resource>) {
        self.resources.insert(as_type, RefCell::new(resource));
    }

    pub fn remove_resource<T>(&mut self)
    where
        T: 'static,
    {
        self.resources.remove(&TypeId::of::<T>());
    }

    pub fn has_resource<T>(&self) -> bool
    where
        T: 'static,
    {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn resource<T>(&self) -> Option<ResRead<T>>
    where
        T: 'static,
    {
        if let Some(res) = self.resources.get(&TypeId::of::<T>()) {
            return Some(ResRead {
                inner: unsafe { std::mem::transmute(res.borrow()) },
                _phantom: PhantomData::default(),
            });
        }
        None
    }

    pub fn resource_mut<T>(&self) -> Option<ResWrite<T>>
    where
        T: 'static,
    {
        if let Some(res) = self.resources.get(&TypeId::of::<T>()) {
            return Some(ResWrite {
                inner: unsafe { std::mem::transmute(res.borrow_mut()) },
                _phantom: PhantomData::default(),
            });
        }
        None
    }

    pub fn expect_resource<T>(&self) -> ResRead<T>
    where
        T: 'static,
    {
        self.resource::<T>()
            .unwrap_or_else(|| panic!("Resource not found: {}", type_name::<T>()))
    }

    pub fn expect_resource_mut<T>(&self) -> ResWrite<T>
    where
        T: 'static,
    {
        self.resource_mut::<T>()
            .unwrap_or_else(|| panic!("Resource not found: {}", type_name::<T>()))
    }

    pub fn query_resources<T>(&self) -> T::Fetch
    where
        T: ResQuery,
    {
        T::fetch(self)
    }

    pub fn is_running(&self) -> bool {
        !self.states.is_empty()
            && self
                .resource::<AppLifeCycle>()
                .map(|lifecycle| lifecycle.running)
                .unwrap_or_default()
    }

    pub fn maintain(&mut self) {
        if self.states.is_empty() {
            return;
        }
        let mut states = std::mem::take(&mut self.states);
        if self.startup {
            states.last_mut().unwrap().on_enter(self);
            self.startup = false;
        }
        let count = states.len() - 1;
        for state in states.iter_mut().take(count) {
            state.on_process_background(self);
        }
        let change = states.last_mut().unwrap().on_process(self);
        match &change {
            StateChange::Pop | StateChange::Swap(_) => {
                let to_delete = if let Some(lifecycle) = self.resource::<AppLifeCycle>() {
                    let token = lifecycle.current_state_token();
                    self.world()
                        .query::<&NonPersistent>()
                        .iter()
                        .filter_map(
                            |(entity, pers)| if pers.0 == token { Some(entity) } else { None },
                        )
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                for entity in to_delete {
                    let _ = self.world_mut().despawn(entity);
                }
            }
            StateChange::Quit => {
                let to_delete = self
                    .world()
                    .query::<&NonPersistent>()
                    .iter()
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>();
                for entity in to_delete {
                    let _ = self.world_mut().despawn(entity);
                }
            }
            _ => {}
        }
        match change {
            StateChange::Push(mut state) => {
                states.last_mut().unwrap().on_pause(self);
                if let Some(mut lifecycle) = self.resource_mut::<AppLifeCycle>() {
                    lifecycle.states_tokens.push(StateToken::new());
                }
                state.on_enter(self);
                states.push(state);
            }
            StateChange::Pop => {
                states.pop().unwrap().on_exit(self);
                if let Some(mut lifecycle) = self.resource_mut::<AppLifeCycle>() {
                    lifecycle.states_tokens.pop();
                }
                if let Some(state) = states.last_mut() {
                    state.on_resume(self);
                }
            }
            StateChange::Swap(mut state) => {
                states.pop().unwrap().on_exit(self);
                if let Some(mut lifecycle) = self.resource_mut::<AppLifeCycle>() {
                    lifecycle.states_tokens.pop();
                    lifecycle.states_tokens.push(StateToken::new());
                }
                state.on_enter(self);
                states.push(state);
            }
            StateChange::Quit => {
                while let Some(mut state) = states.pop() {
                    state.on_exit(self);
                    if let Some(mut lifecycle) = self.resource_mut::<AppLifeCycle>() {
                        lifecycle.states_tokens.pop();
                    }
                }
            }
            _ => {}
        }
        if let Some(mut lifecycle) = self.resource_mut::<AppLifeCycle>() {
            lifecycle.timer.tick();
        }
        let _ = std::mem::replace(&mut self.states, states);
    }
}

pub struct UnsafeScope;

pub struct UnsafeRef<'a, T>(&'a UnsafeScope, &'a T);

impl<'a, T> UnsafeRef<'a, T> {
    /// # Safety
    /// Extending lifetimes is unsafe and when done wrongly can cause undefined behaviour.
    /// Make sure lifetime can be extended to the scope where data behind reference won't be moved.
    pub unsafe fn upgrade(scope: &'a UnsafeScope, v: &T) -> Self {
        Self(scope, std::mem::transmute(v))
    }

    /// # Safety
    /// Extending lifetimes is unsafe and when done wrongly can cause undefined behaviour.
    /// Make sure lifetime can be extended to the scope where data behind reference won't be moved.
    pub unsafe fn read(&self) -> &T {
        self.1
    }
}

pub struct UnsafeMut<'a, T>(&'a UnsafeScope, &'a mut T);

impl<'a, T> UnsafeMut<'a, T> {
    /// # Safety
    /// Extending lifetimes is unsafe and when done wrongly can cause undefined behaviour.
    /// Make sure lifetime can be extended to the scope where data behind reference won't be moved.
    pub unsafe fn upgrade(scope: &'a UnsafeScope, v: &mut T) -> Self {
        Self(scope, std::mem::transmute(v))
    }

    /// # Safety
    /// Extending lifetimes is unsafe and when done wrongly can cause undefined behaviour.
    /// Make sure lifetime can be extended to the scope where data behind reference won't be moved.
    pub unsafe fn read(&self) -> &T {
        self.1
    }

    /// # Safety
    /// Extending lifetimes is unsafe and when done wrongly can cause undefined behaviour.
    /// Make sure lifetime can be extended to the scope where data behind reference won't be moved.
    pub unsafe fn write(&mut self) -> &mut T {
        self.1
    }
}

pub trait ResAccess {}

impl ResAccess for () {}

pub trait ResQuery {
    type Fetch: ResAccess;

    fn fetch(universe: &Universe) -> Self::Fetch;
}

pub struct ResRead<T> {
    inner: Ref<'static, Box<Resource>>,
    _phantom: PhantomData<T>,
}

impl<T> ResAccess for ResRead<T> {}

impl<T> Deref for ResRead<T>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref::<Self::Target>().unwrap()
    }
}

pub struct RefRead<T>(Ref<'static, T>)
where
    T: 'static;

impl<T> ResAccess for RefRead<T> {}

impl<T> Deref for RefRead<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ResWrite<T> {
    inner: RefMut<'static, Box<Resource>>,
    _phantom: PhantomData<T>,
}

impl<T> ResAccess for ResWrite<T> {}

impl<T> Deref for ResWrite<T>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref::<Self::Target>().unwrap()
    }
}

impl<T> DerefMut for ResWrite<T>
where
    T: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.downcast_mut::<Self::Target>().unwrap()
    }
}

pub struct RefWrite<T>(RefMut<'static, T>)
where
    T: 'static;

impl<T> ResAccess for RefWrite<T> {}

impl<T> Deref for RefWrite<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RefWrite<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ResAccess for Option<T> where T: ResAccess {}

impl ResQuery for WorldRef {
    type Fetch = RefRead<World>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        RefRead(unsafe { std::mem::transmute(universe.world()) })
    }
}

impl ResQuery for WorldMut {
    type Fetch = RefWrite<World>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        RefWrite(unsafe { std::mem::transmute(universe.world_mut()) })
    }
}

impl<T> ResQuery for Comp<T> {
    type Fetch = ();

    fn fetch(_: &Universe) -> Self::Fetch {}
}

impl<T> ResQuery for &T
where
    T: 'static,
{
    type Fetch = ResRead<T>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        universe.expect_resource::<T>()
    }
}

impl<T> ResQuery for &mut T
where
    T: 'static,
{
    type Fetch = ResWrite<T>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        universe.expect_resource_mut::<T>()
    }
}

impl<T> ResQuery for Option<&T>
where
    T: 'static,
{
    type Fetch = Option<ResRead<T>>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        universe.resource::<T>()
    }
}

impl<T> ResQuery for Option<&mut T>
where
    T: 'static,
{
    type Fetch = Option<ResWrite<T>>;

    fn fetch(universe: &Universe) -> Self::Fetch {
        universe.resource_mut::<T>()
    }
}

macro_rules! impl_res_query {
    ( $( $ty:ident ),+ ) => {
        impl<$( $ty ),+> ResAccess for ( $( $ty, )+ ) where $( $ty: ResAccess ),+ {}

        impl<$( $ty ),+> ResQuery for ( $( $ty, )+ ) where $( $ty: ResQuery ),+ {
            type Fetch = ( $( $ty::Fetch, )+ );

            fn fetch(universe: &Universe) -> Self::Fetch {
                ( $( $ty::fetch(universe), )+ )
            }
        }
    }
}

impl_res_query!(A);
impl_res_query!(A, B);
impl_res_query!(A, B, C);
impl_res_query!(A, B, C, D);
impl_res_query!(A, B, C, D, E);
impl_res_query!(A, B, C, D, E, F);
impl_res_query!(A, B, C, D, E, F, G);
impl_res_query!(A, B, C, D, E, F, G, H);
impl_res_query!(A, B, C, D, E, F, G, H, I);
impl_res_query!(A, B, C, D, E, F, G, H, I, J);
impl_res_query!(A, B, C, D, E, F, G, H, I, J, K);
impl_res_query!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_res_query!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_res_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_res_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

pub trait AccessType {
    fn feed_types(_reads: &mut HashSet<TypeId>, _writes: &mut HashSet<TypeId>) {}

    /// ([reads], [writes])
    fn get_types() -> (HashSet<TypeId>, HashSet<TypeId>) {
        let mut reads = HashSet::new();
        let mut writes = HashSet::new();
        Self::feed_types(&mut reads, &mut writes);
        (reads, writes)
    }
}

impl AccessType for () {}

impl AccessType for WorldRef {
    fn feed_types(reads: &mut HashSet<TypeId>, _: &mut HashSet<TypeId>) {
        reads.insert(TypeId::of::<World>());
    }
}

impl AccessType for WorldMut {
    fn feed_types(_: &mut HashSet<TypeId>, writes: &mut HashSet<TypeId>) {
        writes.insert(TypeId::of::<World>());
    }
}

impl<T> AccessType for Comp<T>
where
    T: AccessType,
{
    fn feed_types(reads: &mut HashSet<TypeId>, writes: &mut HashSet<TypeId>) {
        T::feed_types(reads, writes);
    }
}

impl<T> AccessType for &T
where
    T: 'static,
{
    fn feed_types(reads: &mut HashSet<TypeId>, _: &mut HashSet<TypeId>) {
        reads.insert(TypeId::of::<T>());
    }
}

impl<T> AccessType for &mut T
where
    T: 'static,
{
    fn feed_types(_: &mut HashSet<TypeId>, writes: &mut HashSet<TypeId>) {
        writes.insert(TypeId::of::<T>());
    }
}

macro_rules! impl_access_type {
    ( $( $ty:ident ),+ ) => {
        impl<$( $ty ),+> AccessType for ( $( $ty, )+ ) where $( $ty: AccessType ),+ {
            fn feed_types(reads: &mut HashSet<TypeId>,writes: &mut HashSet<TypeId>) {
                $( $ty::feed_types(reads, writes); )+
            }
        }
    }
}

impl_access_type!(A);
impl_access_type!(A, B);
impl_access_type!(A, B, C);
impl_access_type!(A, B, C, D);
impl_access_type!(A, B, C, D, E);
impl_access_type!(A, B, C, D, E, F);
impl_access_type!(A, B, C, D, E, F, G);
impl_access_type!(A, B, C, D, E, F, G, H);
impl_access_type!(A, B, C, D, E, F, G, H, I);
impl_access_type!(A, B, C, D, E, F, G, H, I, J);
impl_access_type!(A, B, C, D, E, F, G, H, I, J, K);
impl_access_type!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_access_type!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_access_type!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_access_type!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_sync() {
        fn foo<T: Send + Sync>() {
            println!("{} is Send + Sync", std::any::type_name::<T>());
        }

        foo::<Universe>();
        foo::<Multiverse>();
    }
}
