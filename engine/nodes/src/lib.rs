extern crate oxygengine_core as core;

pub mod __internal {
    pub use core::scripting::{
        intuicio::data::managed::{DynamicManagedRef, DynamicManagedRefMut},
        ScriptFunctionReference,
    };
}

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

pub enum ScriptedNodesParam {
    Ref(DynamicManagedRef),
    RefMut(DynamicManagedRefMut),
    ScopedRef(DynamicManagedRef, Lifetime),
    ScopedRefMut(DynamicManagedRefMut, Lifetime),
}

impl ScriptedNodesParam {
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
        }
    }

    pub fn arg(mut self, data: ScriptedNodesParam) -> Self {
        self.arguments.push(data);
        self
    }

    pub fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub fn dispatch(&self, universe: &Universe) {
        let world = universe.world();
        let mut nodes = universe.expect_resource_mut::<ScriptedNodes>();
        let scripting = universe.expect_resource::<Scripting>();
        let hierarchy = universe.expect_resource::<Hierarchy>();

        if let Some(function) = scripting.registry.find_function(self.function.query()) {
            Self::execute(
                self.entity,
                function,
                &self.arguments,
                self.broadcast,
                &world,
                &mut nodes,
                &scripting,
                &hierarchy,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        entity: Entity,
        function: FunctionHandle,
        args: &[ScriptedNodesParam],
        broadcast: bool,
        world: &World,
        nodes: &mut ScriptedNodes,
        scripting: &Scripting,
        hierarchy: &Hierarchy,
    ) {
        if let Ok(mut node) = world.query_one::<&mut ScriptedNode>(entity) {
            if let Some(node) = node.get() {
                if let Some(handle) = &function.signature().struct_handle {
                    if node.0.type_hash() != &handle.type_hash() {
                        return;
                    }
                }
                for arg in args.iter().rev() {
                    match arg {
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
                if broadcast {
                    if let Some(iter) = hierarchy.children(entity) {
                        for entity in iter {
                            ScriptedNodeSignal::execute(
                                entity,
                                function.clone(),
                                args,
                                broadcast,
                                world,
                                nodes,
                                scripting,
                                hierarchy,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub struct ScriptedNodes {
    context: Context,
    signals: Vec<ScriptedNodeSignal>,
    spawns: Vec<(ScriptedNodesTree, Option<Entity>)>,
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
            signals: Default::default(),
            spawns: Default::default(),
        }
    }

    pub fn signal(&mut self, signal: ScriptedNodeSignal) {
        self.signals.push(signal);
    }

    pub fn spawn(&mut self, tree: ScriptedNodesTree, parent: Option<Entity>) {
        self.spawns.push((tree, parent));
    }

    pub fn maintain(&mut self, universe: &Universe) {
        let world = universe.world();
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();
        let scripting = universe.expect_resource::<Scripting>();
        let hierarchy = universe.expect_resource::<Hierarchy>();

        for signal in std::mem::take(&mut self.signals) {
            if let Some(function) = scripting.registry.find_function(signal.function.query()) {
                ScriptedNodeSignal::execute(
                    signal.entity,
                    function,
                    &signal.arguments,
                    signal.broadcast,
                    &world,
                    self,
                    &scripting,
                    &hierarchy,
                );
            }
        }

        for (tree, parent) in std::mem::take(&mut self.spawns) {
            Self::execute_spawn(tree, parent, &mut commands);
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
        } = tree;
        if let Some(entity) = parent {
            components.add(Parent(entity));
        }
        components.add(ScriptedNode(object));
        commands.schedule(
            SpawnEntity::new(components).on_complete(move |universe, entity| {
                let mut commands = universe.expect_resource_mut::<UniverseCommands>();
                for child in children {
                    Self::execute_spawn((child)(entity), Some(entity), &mut commands);
                }
            }),
        );
    }

    pub fn dispatch(
        &mut self,
        universe: &Universe,
        function: ScriptFunctionReference,
        args: &[ScriptedNodesParam],
    ) {
        let world = universe.world();
        let scripting = universe.expect_resource::<Scripting>();
        let hierarchy = universe.expect_resource::<Hierarchy>();

        if let Some(function) = scripting.registry.find_function(function.query()) {
            for (entity, _) in world
                .query::<()>()
                .with::<&ScriptedNode>()
                .without::<&Parent>()
                .iter()
            {
                self.execute(
                    entity,
                    function.clone(),
                    args,
                    &world,
                    &scripting,
                    &hierarchy,
                )
            }
        }
        self.context
            .stack()
            .restore(unsafe { DataStackToken::new(0) });
    }

    pub fn execute(
        &mut self,
        entity: Entity,
        function: FunctionHandle,
        args: &[ScriptedNodesParam],
        world: &World,
        scripting: &Scripting,
        hierarchy: &Hierarchy,
    ) {
        if let Ok(mut node) = world.query_one::<&mut ScriptedNode>(entity) {
            if let Some(node) = node.get() {
                if let Some(handle) = &function.signature().struct_handle {
                    if node.0.type_hash() != &handle.type_hash() {
                        return;
                    }
                }
                for arg in args.iter().rev() {
                    match arg {
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
                if let Some(iter) = hierarchy.children(entity) {
                    for entity in iter {
                        self.execute(entity, function.clone(), args, world, scripting, hierarchy)
                    }
                }
            }
        }
    }
}

pub struct ScriptedNodesTree {
    data: DynamicManaged,
    components: EntityBuilder,
    children: Vec<Box<dyn FnOnce(Entity) -> Self + Send + Sync>>,
}

impl ScriptedNodesTree {
    pub fn new<T: 'static>(data: T) -> Self {
        Self::new_raw(DynamicManaged::new(data))
    }

    pub fn new_raw(data: DynamicManaged) -> Self {
        Self {
            data,
            children: Default::default(),
            components: Default::default(),
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
}

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    nodes: ScriptedNodes,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(nodes);
    Ok(())
}

#[macro_export]
macro_rules! nodes_dispatch {
    ($nodes:expr, $universe:expr => $function:ident( $($arg:expr),* ) ) => {
        $nodes.dispatch(
            $universe,
            $crate::__internal::ScriptFunctionReference::parse(stringify!($function))
                .unwrap_or_else(|error| panic!(
                    "Could not parse script function reference: {}. Error: {}",
                    stringify!($function),
                    error,
                )),
            &[ $( ($arg).into() ),* ],
        );
    };
}
