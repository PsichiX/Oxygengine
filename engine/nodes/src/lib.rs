extern crate oxygengine_core as core;

use core::{
    app::AppBuilder,
    ecs::{
        commands::{SpawnEntity, UniverseCommands},
        components::Name,
        hierarchy::{Hierarchy, Parent},
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Component, Entity, EntityBuilder, Query, Universe, World,
    },
    scripting::{
        intuicio::{core::prelude::*, data::prelude::*},
        ScriptFunctionReference, Scripting,
    },
};
use std::borrow::Cow;

const DEFAULT_CAPACITY: usize = 10240;

#[derive(Default, Clone)]
pub struct ScriptedNodeEntity(AsyncShared<Option<Entity>>);

impl ScriptedNodeEntity {
    pub fn new(entity: Entity) -> Self {
        Self(AsyncShared::new(Some(entity)))
    }

    pub fn find(path: &str, hierarchy: &Hierarchy) -> Self {
        if let Some(entity) = hierarchy.entity_by_name(path) {
            Self::new(entity)
        } else {
            Self::default()
        }
    }

    pub fn find_raw(path: &str, hierarchy: &Hierarchy) -> Option<Entity> {
        if let Some(entity) = hierarchy.entity_by_name(path) {
            Some(entity)
        } else {
            None
        }
    }

    pub fn find_all_of_type<T: 'static>(world: &World) -> Vec<Self> {
        world
            .query::<&ScriptedNode>()
            .iter()
            .filter_map(move |(entity, node)| {
                if node.is::<T>() {
                    Some(Self::new(entity))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn find_all_of_type_raw<T: 'static>(world: &World) -> Vec<Entity> {
        world
            .query::<&ScriptedNode>()
            .iter()
            .filter_map(
                move |(entity, node)| {
                    if node.is::<T>() {
                        Some(entity)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }

    pub fn set(&mut self, entity: Entity) {
        if let Some(mut data) = self.0.write() {
            *data = Some(entity);
        }
    }

    pub fn get(&self) -> Option<Entity> {
        self.0.read().and_then(|data| data.as_ref().copied())
    }

    pub fn is_valid(&self) -> bool {
        self.0.read().map(|data| data.is_some()).unwrap_or_default()
    }

    pub fn take(&mut self) -> Option<Entity> {
        self.0.write().and_then(|mut data| data.take())
    }

    pub fn node<Q: Query, R>(
        &self,
        world: &World,
        mut f: impl FnMut(&ScriptedNode, Q::Item<'_>) -> R,
    ) -> Option<R> {
        let entity = *self.0.read()?.as_ref()?;
        let mut query = world.query_one::<(&ScriptedNode, Q)>(entity).ok()?;
        let (node, query) = query.get()?;
        Some(f(node, query))
    }

    pub fn node_mut<Q: Query, R>(
        &self,
        world: &World,
        mut f: impl FnMut(&mut ScriptedNode, Q::Item<'_>) -> R,
    ) -> Option<R> {
        let entity = *self.0.read()?.as_ref()?;
        let mut query = world.query_one::<(&mut ScriptedNode, Q)>(entity).ok()?;
        let (node, query) = query.get()?;
        Some(f(node, query))
    }

    pub fn with<T: 'static, Q: Query, R>(
        &self,
        world: &World,
        mut f: impl FnMut(&T, Q::Item<'_>) -> R,
    ) -> Option<R> {
        let entity = *self.0.read()?.as_ref()?;
        let mut query = world.query_one::<(&ScriptedNode, Q)>(entity).ok()?;
        let (node, query) = query.get()?;
        let node = node.read::<T>()?;
        Some(f(&node, query))
    }

    pub fn with_mut<T: 'static, Q: Query, R>(
        &self,
        world: &World,
        mut f: impl FnMut(&mut T, Q::Item<'_>) -> R,
    ) -> Option<R> {
        let entity = *self.0.read()?.as_ref()?;
        let mut query = world.query_one::<(&mut ScriptedNode, Q)>(entity).ok()?;
        let (node, query) = query.get()?;
        let mut node = node.write::<T>()?;
        Some(f(&mut node, query))
    }
}

impl std::fmt::Debug for ScriptedNodeEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct("ScriptedNodeEntity");
        match self.get() {
            Some(entity) => result.field("entity", &entity).finish(),
            None => result.finish_non_exhaustive(),
        }
    }
}

pub enum ScriptedNodesParam {
    Owned(DynamicManaged),
    Ref(DynamicManagedRef),
    RefMut(DynamicManagedRefMut),
    ScopedRef(DynamicManagedRef, Lifetime),
    ScopedRefMut(DynamicManagedRefMut, Lifetime),
}

impl ScriptedNodesParam {
    pub fn owned<T: 'static>(value: T) -> Self {
        Self::Owned(DynamicManaged::new(value))
    }

    pub fn scoped_ref<'a, T: 'static>(value: &'a T) -> Self
    where
        Self: 'a,
    {
        let lifetime = Lifetime::default();
        let data = DynamicManagedRef::new(value, lifetime.borrow().unwrap());
        Self::ScopedRef(data, lifetime)
    }

    pub fn scoped_ref_mut<'a, T: 'static>(value: &'a mut T) -> Self
    where
        Self: 'a,
    {
        let lifetime = Lifetime::default();
        let data = DynamicManagedRefMut::new(value, lifetime.borrow_mut().unwrap());
        Self::ScopedRefMut(data, lifetime)
    }
}

impl From<DynamicManaged> for ScriptedNodesParam {
    fn from(value: DynamicManaged) -> Self {
        Self::Owned(value)
    }
}

impl From<DynamicManagedRef> for ScriptedNodesParam {
    fn from(value: DynamicManagedRef) -> Self {
        Self::Ref(value)
    }
}

impl From<DynamicManagedRefMut> for ScriptedNodesParam {
    fn from(value: DynamicManagedRefMut) -> Self {
        Self::RefMut(value)
    }
}

impl<T: 'static> From<&T> for ScriptedNodesParam {
    fn from(value: &T) -> Self {
        Self::scoped_ref(value)
    }
}

impl<T: 'static> From<&mut T> for ScriptedNodesParam {
    fn from(value: &mut T) -> Self {
        Self::scoped_ref_mut(value)
    }
}

pub struct ScriptedNode {
    pub active: bool,
    pub object: DynamicManaged,
}

impl ScriptedNode {
    pub fn new<T: 'static>(data: T) -> Self {
        Self::new_raw(DynamicManaged::new(data))
    }

    pub fn new_raw(object: DynamicManaged) -> Self {
        Self {
            active: true,
            object,
        }
    }

    pub fn with_active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.object.is::<T>()
    }

    pub fn read<T: 'static>(&self) -> Option<ValueReadAccess<T>> {
        self.object.read::<T>()
    }

    pub fn write<T: 'static>(&mut self) -> Option<ValueWriteAccess<T>> {
        self.object.write::<T>()
    }
}

pub struct ScriptedNodeSignal {
    entity: Option<Entity>,
    function: ScriptFunctionReference,
    arguments: Vec<ScriptedNodesParam>,
    broadcast: bool,
    bubble: bool,
    ignore_me: bool,
}

impl ScriptedNodeSignal {
    pub fn parse(entity: Option<Entity>, content: &str) -> Result<Self, String> {
        Ok(Self::new(entity, ScriptFunctionReference::parse(content)?))
    }

    pub fn new(entity: Option<Entity>, function: ScriptFunctionReference) -> Self {
        Self {
            entity,
            function,
            arguments: Default::default(),
            broadcast: false,
            bubble: false,
            ignore_me: false,
        }
    }

    pub fn arg(mut self, data: impl Into<ScriptedNodesParam>) -> Self {
        self.arguments.push(data.into());
        self
    }

    pub fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub fn bubble(mut self) -> Self {
        self.bubble = true;
        self
    }

    pub fn ignore_me(mut self) -> Self {
        self.ignore_me = true;
        self
    }

    pub fn dispatch<T: ScriptedNodeComponentPack>(&self, universe: &Universe) {
        let world = universe.world();
        let mut nodes = universe.expect_resource_mut::<ScriptedNodes>();
        let scripting = universe.expect_resource::<Scripting>();
        let hierarchy = universe.expect_resource::<Hierarchy>();

        if let Some(entity) = self.entity {
            Self::execute::<T>(
                entity,
                &self.function,
                &self.arguments,
                self.broadcast,
                self.bubble,
                self.ignore_me,
                &world,
                &mut nodes,
                &scripting,
                &hierarchy,
            );
        } else {
            for (entity, _) in world
                .query::<()>()
                .with::<&ScriptedNode>()
                .without::<&Parent>()
                .iter()
            {
                Self::execute::<T>(
                    entity,
                    &self.function,
                    &self.arguments,
                    self.broadcast,
                    self.bubble,
                    self.ignore_me,
                    &world,
                    &mut nodes,
                    &scripting,
                    &hierarchy,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute<T: ScriptedNodeComponentPack>(
        entity: Entity,
        function_ref: &ScriptFunctionReference,
        args: &[ScriptedNodesParam],
        broadcast: bool,
        bubble: bool,
        ignore_me: bool,
        world: &World,
        nodes: &mut ScriptedNodes,
        scripting: &Scripting,
        hierarchy: &Hierarchy,
    ) {
        let token = nodes.context.stack().store();
        let result = if !ignore_me {
            if let Ok(mut query) = world.query_one::<(&mut ScriptedNode, T)>(entity) {
                if let Some((node, pack)) = query.get() {
                    let mut query = function_ref.query();
                    if query.struct_query.is_none() {
                        query.struct_query = Some(StructQuery {
                            type_hash: Some(*node.object.type_hash()),
                            ..Default::default()
                        });
                    }
                    if let Some(function) = scripting.registry.find_function(query) {
                        if let Some(handle) = &function.signature().struct_handle {
                            if node.object.type_hash() != &handle.type_hash() {
                                return;
                            }
                        }
                        nodes.context.stack().push(DynamicManaged::new(entity));
                        let mut compontents_params = vec![];
                        T::query_param(pack, &mut compontents_params);
                        for arg in compontents_params.iter().chain(args.iter()).rev() {
                            match arg {
                                ScriptedNodesParam::Owned(arg) => {
                                    nodes.context.stack().push(arg.borrow().unwrap());
                                }
                                ScriptedNodesParam::Ref(arg) => {
                                    nodes.context.stack().push(arg.borrow().unwrap());
                                }
                                ScriptedNodesParam::RefMut(arg) => {
                                    nodes.context.stack().push(arg.borrow_mut().unwrap());
                                }
                                ScriptedNodesParam::ScopedRef(arg, _) => {
                                    nodes.context.stack().push(arg.borrow().unwrap());
                                }
                                ScriptedNodesParam::ScopedRefMut(arg, _) => {
                                    nodes.context.stack().push(arg.borrow_mut().unwrap());
                                }
                            }
                        }
                        nodes
                            .context
                            .stack()
                            .push(node.object.borrow_mut().unwrap());
                        Some((function, compontents_params))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some((function, _)) = result {
            function.invoke(&mut nodes.context, &scripting.registry);
        }
        nodes.context.stack().restore(token);
        if broadcast {
            if let Some(iter) = hierarchy.children(entity) {
                for entity in iter {
                    Self::execute::<T>(
                        entity,
                        function_ref,
                        args,
                        true,
                        false,
                        false,
                        world,
                        nodes,
                        scripting,
                        hierarchy,
                    );
                }
            }
        }
        if bubble {
            if let Some(entity) = hierarchy.parent(entity) {
                Self::execute::<T>(
                    entity,
                    function_ref,
                    args,
                    false,
                    true,
                    false,
                    world,
                    nodes,
                    scripting,
                    hierarchy,
                );
            }
        }
    }
}

#[derive(Default)]
pub struct ScriptedNodesSpawns {
    spawns: Vec<(ScriptedNodesTree, Option<Entity>)>,
}

impl ScriptedNodesSpawns {
    pub fn spawn(&mut self, tree: ScriptedNodesTree, parent: Option<Entity>) {
        self.spawns.push((tree, parent));
    }

    pub fn spawn_root(&mut self, tree: ScriptedNodesTree) {
        self.spawn(tree, None)
    }
}

#[derive(Default)]
pub struct ScriptedNodesSignals {
    #[allow(clippy::type_complexity)]
    signals: Vec<Box<dyn FnOnce(&Universe) + Send + Sync>>,
}

impl ScriptedNodesSignals {
    pub fn signal<T: ScriptedNodeComponentPack>(&mut self, signal: ScriptedNodeSignal) {
        self.signals
            .push(Box::new(move |universe| signal.dispatch::<T>(universe)));
    }
}

pub struct ScriptedNodes {
    context: Context,
}

impl Default for ScriptedNodes {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY, DEFAULT_CAPACITY, DEFAULT_CAPACITY)
    }
}

impl ScriptedNodes {
    pub fn new(
        stack_capacity: usize,
        registers_capacity: usize,
        heap_page_capacity: usize,
    ) -> Self {
        Self {
            context: Context::new(stack_capacity, registers_capacity, heap_page_capacity),
        }
    }

    pub fn maintain(universe: &Universe) {
        {
            let mut signals = universe.expect_resource_mut::<ScriptedNodesSignals>();
            for signal in std::mem::take(&mut signals.signals) {
                signal(universe);
            }
        }
        {
            let mut commands = universe.expect_resource_mut::<UniverseCommands>();
            let mut spawns = universe.expect_resource_mut::<ScriptedNodesSpawns>();
            for (tree, parent) in std::mem::take(&mut spawns.spawns) {
                Self::execute_spawn(tree, parent, &mut commands);
            }
        }
    }

    fn execute_spawn(
        tree: ScriptedNodesTree,
        parent: Option<Entity>,
        commands: &mut UniverseCommands,
    ) {
        let ScriptedNodesTree {
            active,
            object,
            children,
            mut components,
            setup,
            bind,
        } = tree;
        if let Some(entity) = parent {
            components.add(Parent(entity));
        }
        components.add(ScriptedNode { active, object });
        commands.schedule(
            SpawnEntity::new(components).on_complete(move |universe, entity| {
                if let Some(function) = setup {
                    let mut signals = universe.expect_resource_mut::<ScriptedNodesSignals>();
                    signals.signal::<()>(ScriptedNodeSignal::new(Some(entity), function));
                }
                if let Some(bind) = bind {
                    (bind)(entity);
                }
                let mut commands = universe.expect_resource_mut::<UniverseCommands>();
                for child in children {
                    Self::execute_spawn((child)(), Some(entity), &mut commands);
                }
            }),
        );
    }

    pub fn dispatch<T: ScriptedNodeComponentPack>(
        &mut self,
        universe: &Universe,
        function: ScriptFunctionReference,
        args: &[ScriptedNodesParam],
    ) {
        let world = universe.world();
        let scripting = universe.expect_resource::<Scripting>();
        let hierarchy = universe.expect_resource::<Hierarchy>();

        for (entity, _) in world
            .query::<()>()
            .with::<&ScriptedNode>()
            .without::<&Parent>()
            .iter()
        {
            self.execute::<T>(entity, &function, args, &world, &scripting, &hierarchy);
        }
    }

    pub fn execute<T: ScriptedNodeComponentPack>(
        &mut self,
        entity: Entity,
        function_ref: &ScriptFunctionReference,
        args: &[ScriptedNodesParam],
        world: &World,
        scripting: &Scripting,
        hierarchy: &Hierarchy,
    ) {
        if let Ok(mut query) = world.query_one::<&ScriptedNode>(entity) {
            if let Some(node) = query.get() {
                if !node.active {
                    return;
                }
            }
        }
        let token = self.context.stack().store();
        let result = if let Ok(mut query) = world.query_one::<(&mut ScriptedNode, T)>(entity) {
            if let Some((node, pack)) = query.get() {
                let mut query = function_ref.query();
                if query.struct_query.is_none() {
                    query.struct_query = Some(StructQuery {
                        type_hash: Some(*node.object.type_hash()),
                        ..Default::default()
                    });
                }
                if let Some(function) = scripting.registry.find_function(query) {
                    if let Some(handle) = &function.signature().struct_handle {
                        if node.object.type_hash() != &handle.type_hash() {
                            return;
                        }
                    }
                    self.context.stack().push(DynamicManaged::new(entity));
                    let mut compontents_params = vec![];
                    T::query_param(pack, &mut compontents_params);
                    for arg in compontents_params.iter().chain(args.iter()).rev() {
                        match arg {
                            ScriptedNodesParam::Owned(arg) => {
                                self.context.stack().push(arg.borrow().unwrap());
                            }
                            ScriptedNodesParam::Ref(arg) => {
                                self.context.stack().push(arg.borrow().unwrap());
                            }
                            ScriptedNodesParam::RefMut(arg) => {
                                self.context.stack().push(arg.borrow_mut().unwrap());
                            }
                            ScriptedNodesParam::ScopedRef(arg, _) => {
                                self.context.stack().push(arg.borrow().unwrap());
                            }
                            ScriptedNodesParam::ScopedRefMut(arg, _) => {
                                self.context.stack().push(arg.borrow_mut().unwrap());
                            }
                        }
                    }
                    self.context.stack().push(node.object.borrow_mut().unwrap());
                    Some((function, compontents_params))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some((function, _)) = result {
            function.invoke(&mut self.context, &scripting.registry);
        }
        self.context.stack().restore(token);
        if let Some(iter) = hierarchy.children(entity) {
            for entity in iter {
                self.execute::<T>(entity, function_ref, args, &world, scripting, hierarchy);
            }
        }
    }
}

pub struct ScriptedNodesTree {
    active: bool,
    object: DynamicManaged,
    components: EntityBuilder,
    children: Vec<Box<dyn FnOnce() -> Self + Send + Sync>>,
    setup: Option<ScriptFunctionReference>,
    bind: Option<Box<dyn FnOnce(Entity) + Send + Sync>>,
}

impl ScriptedNodesTree {
    pub fn empty() -> Self {
        Self::new(())
    }

    pub fn new<T: 'static>(data: T) -> Self {
        Self::new_raw(DynamicManaged::new(data))
    }

    pub fn new_raw(object: DynamicManaged) -> Self {
        Self {
            active: true,
            object,
            children: Default::default(),
            components: Default::default(),
            setup: None,
            bind: None,
        }
    }

    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.components.add(Name(name.into()));
        self
    }

    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }

    pub fn component<T: Component>(mut self, component: T) -> Self {
        self.components.add(component);
        self
    }

    pub fn child(mut self, f: impl FnOnce() -> Self + Send + Sync + 'static) -> Self {
        self.children.push(Box::new(f));
        self
    }

    pub fn setup(mut self, function: ScriptFunctionReference) -> Self {
        self.setup = Some(function);
        self
    }

    pub fn bind(mut self, f: impl FnOnce(Entity) + Send + Sync + 'static) -> Self {
        self.bind = Some(Box::new(f));
        self
    }
}

pub trait ScriptedNodeComponentPack: Query {
    fn query_param(pack: Self::Item<'_>, list: &mut Vec<ScriptedNodesParam>);
}

impl ScriptedNodeComponentPack for () {
    fn query_param(_: (), _: &mut Vec<ScriptedNodesParam>) {}
}

impl<T: Component> ScriptedNodeComponentPack for &T {
    fn query_param(pack: Self::Item<'_>, list: &mut Vec<ScriptedNodesParam>) {
        list.push(ScriptedNodesParam::scoped_ref(pack));
    }
}

impl<T: Component> ScriptedNodeComponentPack for &mut T {
    fn query_param(pack: Self::Item<'_>, list: &mut Vec<ScriptedNodesParam>) {
        list.push(ScriptedNodesParam::scoped_ref_mut(pack));
    }
}

macro_rules! impl_component_tuple {
    ($($type:ident),+) => {
        impl<$($type: ScriptedNodeComponentPack),+> ScriptedNodeComponentPack for ($($type,)+) {
            fn query_param(pack: Self::Item<'_>, list: &mut Vec<ScriptedNodesParam>) {
                #[allow(non_snake_case)]
                let ( $($type,)+ ) = pack;
                $(
                    $type::query_param($type, list);
                )+
            }
        }
    };
}

impl_component_tuple!(A);
impl_component_tuple!(A, B);
impl_component_tuple!(A, B, C);
impl_component_tuple!(A, B, C, D);
impl_component_tuple!(A, B, C, D, E);
impl_component_tuple!(A, B, C, D, E, F);
impl_component_tuple!(A, B, C, D, E, F, G);
impl_component_tuple!(A, B, C, D, E, F, G, H);
impl_component_tuple!(A, B, C, D, E, F, G, H, I);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    nodes: ScriptedNodes,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(nodes);
    builder.install_resource(ScriptedNodesSpawns::default());
    builder.install_resource(ScriptedNodesSignals::default());
    Ok(())
}
