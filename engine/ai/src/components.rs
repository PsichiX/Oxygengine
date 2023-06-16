use crate::resources::AiBehaviorMemory;
use emergent::prelude::*;
use oxygengine_core::{
    ecs::Component,
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct AiInstance<C>
where
    C: Component,
{
    pub template: String,
    #[serde(default)]
    pub decide_delay: Scalar,
    #[serde(skip)]
    pub(crate) decide_phase: Scalar,
    #[serde(skip)]
    pub(crate) decision_maker: Option<Box<dyn DecisionMakingTask<AiBehaviorMemory<C>>>>,
}

impl<C> AiInstance<C>
where
    C: Component,
{
    pub fn new(template: impl ToString, decide_delay: Scalar) -> Self {
        Self {
            template: template.to_string(),
            decide_delay,
            decide_phase: 0.0,
            decision_maker: None,
        }
    }
}

impl<C> Prefab for AiInstance<C> where C: Component + Default {}

impl<C> PrefabComponent for AiInstance<C> where C: Component + Default {}
