use crate::{
    components::AiInstance,
    resources::{AiBehaviorMemory, AiBehaviors},
};
use oxygengine_core::{
    app::AppLifeCycle,
    ecs::{commands::UniverseCommands, Comp, Component, Universe, WorldRef},
};

pub type AiSystemResources<'a, C> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AiBehaviors<C>,
    &'a mut UniverseCommands,
    Comp<&'a mut C>,
    Comp<&'a mut AiInstance<C>>,
);

pub fn ai_system<C>(universe: &mut Universe)
where
    C: Component,
{
    let (world, lifecycle, behaviors, mut commands, ..) =
        universe.query_resources::<AiSystemResources<C>>();

    let dt = lifecycle.delta_time_seconds();

    for (entity, (mut component, instance)) in world.query::<(&mut C, &mut AiInstance<C>)>().iter()
    {
        if let Some(decision_maker) = &mut instance.decision_maker {
            instance.decide_phase -= dt;
            if instance.decide_phase <= 0.0 {
                instance.decide_phase = instance.decide_delay;
                let mut memory = AiBehaviorMemory {
                    entity,
                    component: unsafe { std::mem::transmute(&mut component) },
                    commands: unsafe { std::mem::transmute(&mut commands) },
                };
                decision_maker.decide(&mut memory);
            }
        } else if let Ok(mut decision_maker) = behaviors.instantiate(&instance.template) {
            let mut memory = AiBehaviorMemory {
                entity,
                component: unsafe { std::mem::transmute(&mut component) },
                commands: unsafe { std::mem::transmute(&mut commands) },
            };
            decision_maker.decide(&mut memory);
        }
    }
}
