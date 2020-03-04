use crate::resource::VnStoryManager;
#[cfg(feature = "script-flow")]
use crate::story::ScriptFlowEvent;
use core::{
    app::AppLifeCycle,
    ecs::{ReadExpect, System, Write},
    Scalar,
};
#[cfg(feature = "script-flow")]
use flow::{resource::FlowManager, vm::Reference};

#[derive(Default)]
pub struct VnStorySystem;

impl<'s> System<'s> for VnStorySystem {
    type SystemData = (ReadExpect<'s, AppLifeCycle>, Write<'s, VnStoryManager>);

    fn run(&mut self, (lifecycle, mut manager): Self::SystemData) {
        manager.process(lifecycle.delta_time_seconds() as Scalar);
    }
}

#[cfg(feature = "script-flow")]
pub struct VnScriptFlowSystem;

#[cfg(feature = "script-flow")]
impl<'s> System<'s> for VnScriptFlowSystem {
    type SystemData = (Write<'s, VnStoryManager>, Write<'s, FlowManager>);

    fn run(&mut self, (mut stories, mut flow): Self::SystemData) {
        for story in stories.stories_mut() {
            match story.replace_script_flow_event(ScriptFlowEvent::None) {
                ScriptFlowEvent::Call(vm_name, event_name, parameters) => {
                    if let Some(vm) = flow.get_mut(&vm_name) {
                        let references = parameters
                            .into_iter()
                            .map(|v| Reference::value(v.into()))
                            .collect::<Vec<_>>();
                        if let Ok(guid) = vm.run_event(&event_name, references) {
                            story.replace_script_flow_event(ScriptFlowEvent::InProgress(guid));
                        }
                    }
                }
                ScriptFlowEvent::InProgress(guid) => {
                    for vm in flow.vms_mut() {
                        if let Some(references) = vm.get_completed_event(guid) {
                            let values = references
                                .into_iter()
                                .map(|r| r.into_inner().into())
                                .collect::<Vec<_>>();
                            story.replace_script_flow_event(ScriptFlowEvent::Done(values));
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
