use emergent::prelude::*;
use oxygengine_core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    id::ID,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub use emergent;

#[derive(Debug, Clone)]
pub enum AiBehaviorError {
    TemplateDoesNotExists(String),
    BehaviorDoesNotExists(AiBehaviorId),
    TryingToAccessWrongMemoryType(TypeId),
}

pub type AiBehaviorId = ID<AiBehavior<()>>;

pub struct AiBehavior<M> {
    inner: Box<dyn DecisionMakingTask<M>>,
}

impl<M> AiBehavior<M> {
    pub fn new<D>(mut decision_making_task: D, memory: &mut M) -> Self
    where
        D: DecisionMakingTask<M> + 'static,
    {
        decision_making_task.on_enter(memory);
        Self {
            inner: Box::new(decision_making_task),
        }
    }

    pub fn consume(mut self, memory: &mut M) {
        self.inner.on_exit(memory);
    }

    pub fn is_locked(&self, memory: &M) -> bool {
        self.inner.is_locked(memory)
    }

    pub fn decide(&mut self, memory: &mut M) -> Option<String> {
        self.inner.decide(memory)
    }

    pub fn change_mind(&mut self, id: Option<String>, memory: &mut M) -> bool {
        self.inner.change_mind(id, memory)
    }

    pub fn update(&mut self, memory: &mut M) {
        self.inner.on_update(memory);
    }
}

#[derive(Default)]
pub struct AiBehaviors {
    templates: HashMap<String, Box<dyn Fn() -> Box<dyn Any + Send + Sync> + Send + Sync>>,
    behaviors: HashMap<AiBehaviorId, Box<dyn Any + Send + Sync>>,
}

impl AiBehaviors {
    pub fn add_template<M, F>(&mut self, name: impl ToString, f: F)
    where
        M: 'static,
        F: Fn() -> AiBehavior<M> + Send + Sync + 'static,
    {
        self.templates
            .insert(name.to_string(), Box::new(move || Box::new(f())));
    }

    pub fn with_template<M, F>(mut self, name: impl ToString, f: F) -> Self
    where
        M: 'static,
        F: Fn() -> AiBehavior<M> + Send + Sync + 'static,
    {
        self.add_template(name, f);
        self
    }

    pub fn remove_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    pub fn create<M: 'static>(&mut self, behavior: AiBehavior<M>) -> AiBehaviorId {
        let id = AiBehaviorId::new();
        self.behaviors.insert(id, Box::new(behavior));
        id
    }

    pub fn instantiate(&mut self, name: &str) -> Result<AiBehaviorId, AiBehaviorError> {
        match self.templates.get(name) {
            Some(factory) => {
                let id = AiBehaviorId::new();
                self.behaviors.insert(id, factory());
                Ok(id)
            }
            None => Err(AiBehaviorError::TemplateDoesNotExists(name.to_owned())),
        }
    }

    pub fn destroy<M: 'static>(&mut self, id: AiBehaviorId) -> Option<AiBehavior<M>> {
        self.behaviors
            .remove(&id)
            .and_then(|behavior| behavior.downcast::<AiBehavior<M>>().ok())
            .map(|behavior| *behavior)
    }

    pub fn destroy_consumed<M: 'static>(&mut self, id: AiBehaviorId, memory: &mut M) {
        if let Some(behavior) = self.destroy(id) {
            behavior.consume(memory);
        }
    }

    pub fn get<M: 'static>(&self, id: AiBehaviorId) -> Result<&AiBehavior<M>, AiBehaviorError> {
        match self.behaviors.get(&id) {
            Some(behavior) => match behavior.downcast_ref::<AiBehavior<M>>() {
                Some(behavior) => Ok(behavior),
                None => Err(AiBehaviorError::TryingToAccessWrongMemoryType(
                    TypeId::of::<M>(),
                )),
            },
            None => Err(AiBehaviorError::BehaviorDoesNotExists(id)),
        }
    }

    pub fn get_mut<M: 'static>(
        &mut self,
        id: AiBehaviorId,
    ) -> Result<&mut AiBehavior<M>, AiBehaviorError> {
        match self.behaviors.get_mut(&id) {
            Some(behavior) => match behavior.downcast_mut::<AiBehavior<M>>() {
                Some(behavior) => Ok(behavior),
                None => Err(AiBehaviorError::TryingToAccessWrongMemoryType(
                    TypeId::of::<M>(),
                )),
            },
            None => Err(AiBehaviorError::BehaviorDoesNotExists(id)),
        }
    }
}

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(AiBehaviors::default());

    Ok(())
}
