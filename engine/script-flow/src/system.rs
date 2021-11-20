use crate::resource::FlowScriptManager;
use core::ecs::Universe;

pub type FlowScriptSystemResources<'a> = &'a mut FlowScriptManager;

pub fn flow_script_system(universe: &mut Universe) {
    let _ = universe
        .query_resources::<FlowScriptSystemResources>()
        .process_events();
}
