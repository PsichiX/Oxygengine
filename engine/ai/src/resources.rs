use emergent::prelude::*;
use oxygengine_core::ecs::{
    commands::UniverseCommands,
    commands::{EntityAddComponent, EntityRemoveComponent},
    Component, Entity,
};
use std::{collections::HashMap, marker::PhantomData};

pub type AiBehavior<C> = dyn DecisionMakingTask<AiBehaviorMemory<C>>;

pub type AiCondition<C> = dyn Condition<AiBehaviorMemory<C>>;

pub type AiConsideration<C> = dyn Consideration<AiBehaviorMemory<C>>;

#[derive(Debug, Clone)]
pub enum AiBehaviorError {
    TemplateDoesNotExists(String),
}

pub struct AiBehaviorMemory<C>
where
    C: Component,
{
    pub entity: Entity,
    pub component: &'static mut C,
    pub commands: &'static mut UniverseCommands,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct AiBehaviorTask<MC, TC>(PhantomData<fn() -> (MC, TC)>)
where
    MC: Component,
    TC: Component + Default;

impl<MC, TC> Task<AiBehaviorMemory<MC>> for AiBehaviorTask<MC, TC>
where
    MC: Component,
    TC: Component + Default,
{
    fn on_enter(&mut self, memory: &mut AiBehaviorMemory<MC>) {
        memory
            .commands
            .schedule(EntityAddComponent::new(memory.entity, TC::default()));
    }

    fn on_exit(&mut self, memory: &mut AiBehaviorMemory<MC>) {
        memory
            .commands
            .schedule(EntityRemoveComponent::<TC>::new(memory.entity));
    }
}

type AiBehaviorFactory<C> = Box<dyn Fn() -> Box<AiBehavior<C>> + Send + Sync>;

#[derive(Default)]
pub struct AiBehaviors<C>
where
    C: Component,
{
    templates: HashMap<String, AiBehaviorFactory<C>>,
}

impl<C> AiBehaviors<C>
where
    C: Component,
{
    pub fn add_template<F: 'static>(&mut self, name: impl ToString, f: F)
    where
        F: Fn() -> Box<dyn DecisionMakingTask<AiBehaviorMemory<C>>> + Send + Sync,
    {
        self.templates.insert(name.to_string(), Box::new(f));
    }

    pub fn with_template<F: 'static>(mut self, name: impl ToString, f: F) -> Self
    where
        F: Fn() -> Box<dyn DecisionMakingTask<AiBehaviorMemory<C>>> + Send + Sync,
    {
        self.add_template(name, f);
        self
    }

    pub fn remove_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    pub(crate) fn instantiate(
        &self,
        name: &str,
    ) -> Result<Box<dyn DecisionMakingTask<AiBehaviorMemory<C>>>, AiBehaviorError> {
        match self.templates.get(name) {
            Some(factory) => Ok(factory()),
            None => Err(AiBehaviorError::TemplateDoesNotExists(name.to_owned())),
        }
    }
}
