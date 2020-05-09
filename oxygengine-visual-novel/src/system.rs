#[cfg(feature = "script-flow")]
use crate::story::ScriptFlowEvent;
use crate::{resource::VnStoryManager, vn_story_asset_protocol::VnStoryAsset};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetID, database::AssetsDatabase},
    ecs::{ReadExpect, System, Write},
    Scalar,
};
#[cfg(feature = "script-flow")]
use flow::{resource::FlowManager, vm::Reference};
use std::collections::HashMap;

#[derive(Default)]
pub struct VnStorySystem {
    story_table: HashMap<AssetID, String>,
}

impl<'s> System<'s> for VnStorySystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        ReadExpect<'s, AssetsDatabase>,
        Write<'s, VnStoryManager>,
    );

    fn run(&mut self, (lifecycle, assets, mut manager): Self::SystemData) {
        for id in assets.lately_loaded_protocol("vn-story") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded visual novel story asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<VnStoryAsset>()
                .expect("trying to use non visual novel story asset");
            let story = asset.get().clone();
            manager.register_story(&path, story);
            self.story_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("vn-story") {
            if let Some(path) = self.story_table.remove(id) {
                manager.unregister_story(&path);
            }
        }

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
