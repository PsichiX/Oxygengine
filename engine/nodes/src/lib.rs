extern crate oxygengine_core as core;

use core::{
    app::AppBuilder,
    ecs::{
        commands::{SpawnEntity, UniverseCommands},
        components::{Name, NonPersistent},
        hierarchy::{Hierarchy, Parent},
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Component, Entity, EntityBuilder, Universe, World,
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
    pub fn set(&mut self, entity: Entity) {
        if let Some(mut data) = self.0.write() {
            *data = Some(entity);
        }
    }

    pub fn get(&self) -> Option<Entity> {
        self.0.read().and_then(|data| data.as_ref().copied())
    }

    pub fn take(&mut self) -> Option<Entity> {
        self.0.write().and_then(|mut data| data.take())
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

    pub fn scoped_ref<T: 'static>(value: &T) -> Self {
        let lifetime = Lifetime::default();
        let data = DynamicManagedRef::new(value, lifetime.borrow().unwrap());
        Self::ScopedRef(data, lifetime)
    }

    pub fn scoped_ref_mut<T: 'static>(value: &mut T) -> Self {
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

pub struct ScriptedNode(pub DynamicManaged);

pub struct ScriptedNodeSignal {
    entity: Entity,
    function: ScriptFunctionReference,
    arguments: Vec<ScriptedNodesParam>,
    broadcast: bool,
    bubble: bool,
    ignore_me: bool,
}

impl ScriptedNodeSignal {
    pub fn parse(entity: Entity, content: &str) -> Result<Self, String> {
        Ok(Self::new(entity, ScriptFunctionReference::parse(content)?))
    }

    pub fn new(entity: Entity, function: ScriptFunctionReference) -> Self {
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

        let token = nodes.context.stack().store();
        Self::execute::<T>(
            self.entity,
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
        nodes.context.stack().restore(token);
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
        if let Ok(mut node) = world.query_one::<&mut ScriptedNode>(entity) {
            if let Some(node) = node.get() {
                let mut query = function_ref.query();
                if query.struct_query.is_none() {
                    query.struct_query = Some(StructQuery {
                        type_hash: Some(*node.0.type_hash()),
                        ..Default::default()
                    });
                }
                if !ignore_me {
                    if let Some(function) = scripting.registry.find_function(query) {
                        if let Some(handle) = &function.signature().struct_handle {
                            if node.0.type_hash() != &handle.type_hash() {
                                return;
                            }
                        }
                        let mut compontents_params = vec![];
                        if !T::query_param(entity, world, &mut compontents_params) {
                            return;
                        }
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
                        nodes.context.stack().push(node.0.borrow_mut().unwrap());
                        function.invoke(&mut nodes.context, &scripting.registry);
                    }
                }
            }
        }
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
    pub fn install(registry: &mut Registry) {
        registry.add_struct(NativeStructBuilder::new_uninitialized::<DynamicManaged>().build());
        registry.add_struct(NativeStructBuilder::new_uninitialized::<DynamicManagedRef>().build());
        registry
            .add_struct(NativeStructBuilder::new_uninitialized::<DynamicManagedRefMut>().build());
    }

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
            data: object,
            children,
            mut components,
            setup,
            bind,
        } = tree;
        if let Some(entity) = parent {
            components.add(Parent(entity));
        }
        components.add(ScriptedNode(object));
        commands.schedule(
            SpawnEntity::new(components).on_complete(move |universe, entity| {
                if let Some(function) = setup {
                    let mut signals = universe.expect_resource_mut::<ScriptedNodesSignals>();
                    signals.signal::<()>(ScriptedNodeSignal::new(entity, function));
                }
                if let Some(bind) = bind {
                    (bind)(entity);
                }
                let mut commands = universe.expect_resource_mut::<UniverseCommands>();
                for child in children {
                    Self::execute_spawn((child)(entity), Some(entity), &mut commands);
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

        let token = self.context.stack().store();
        for (entity, _) in world
            .query::<()>()
            .with::<&ScriptedNode>()
            .without::<&Parent>()
            .iter()
        {
            self.execute::<T>(entity, &function, args, &world, &scripting, &hierarchy)
        }
        self.context.stack().restore(token);
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
        if let Ok(mut node) = world.query_one::<&mut ScriptedNode>(entity) {
            if let Some(node) = node.get() {
                let mut query = function_ref.query();
                if query.struct_query.is_none() {
                    query.struct_query = Some(StructQuery {
                        type_hash: Some(*node.0.type_hash()),
                        ..Default::default()
                    });
                }
                if let Some(function) = scripting.registry.find_function(query) {
                    if let Some(handle) = &function.signature().struct_handle {
                        if node.0.type_hash() != &handle.type_hash() {
                            return;
                        }
                    }
                    let mut compontents_params = vec![];
                    if !T::query_param(entity, world, &mut compontents_params) {
                        return;
                    }
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
                    self.context.stack().push(node.0.borrow_mut().unwrap());
                    function.invoke(&mut self.context, &scripting.registry);
                }
            }
        }
        if let Some(iter) = hierarchy.children(entity) {
            for entity in iter {
                self.execute::<T>(entity, function_ref, args, world, scripting, hierarchy)
            }
        }
    }
}

pub struct ScriptedNodesTree {
    data: DynamicManaged,
    components: EntityBuilder,
    children: Vec<Box<dyn FnOnce(Entity) -> Self + Send + Sync>>,
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

    pub fn new_raw(data: DynamicManaged) -> Self {
        Self {
            data,
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

    pub fn non_persistent(mut self) -> Self {
        self.components.add(NonPersistent);
        self
    }

    pub fn component<T: Component>(mut self, component: T) -> Self {
        self.components.add(component);
        self
    }

    pub fn child(mut self, f: impl FnOnce(Entity) -> Self + Send + Sync + 'static) -> Self {
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

pub trait ScriptedNodeComponentPack: Sized {
    fn query_param(entity: Entity, world: &World, list: &mut Vec<ScriptedNodesParam>) -> bool;
}

impl ScriptedNodeComponentPack for () {
    fn query_param(_: Entity, _: &World, _: &mut Vec<ScriptedNodesParam>) -> bool {
        true
    }
}

impl<T: Component> ScriptedNodeComponentPack for &T {
    fn query_param(entity: Entity, world: &World, list: &mut Vec<ScriptedNodesParam>) -> bool {
        if let Ok(mut query) = world.query_one::<&T>(entity) {
            if let Some(component) = query.get() {
                list.push(ScriptedNodesParam::scoped_ref(component));
                return true;
            }
        }
        false
    }
}

impl<T: Component> ScriptedNodeComponentPack for &mut T {
    fn query_param(entity: Entity, world: &World, list: &mut Vec<ScriptedNodesParam>) -> bool {
        if let Ok(mut query) = world.query_one::<&mut T>(entity) {
            if let Some(component) = query.get() {
                list.push(ScriptedNodesParam::scoped_ref_mut(component));
                return true;
            }
        }
        false
    }
}

macro_rules! impl_component_tuple {
    ($($type:ident),+) => {
        impl<$($type: ScriptedNodeComponentPack),+> ScriptedNodeComponentPack for ($($type,)+) {
            #[allow(non_snake_case)]
            fn query_param(entity: Entity, world: &World, list: &mut Vec<ScriptedNodesParam>) -> bool {
                $(
                    if !$type::query_param(entity, world, list) {
                        return false;
                    }
                )+
                true
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
impl_component_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

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
