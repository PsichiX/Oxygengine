use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<VnRenderingManager>()
            .select_config("vn-ui-config.yaml")
            .expect("Could not select config");
        world
            .write_resource::<VnStoryManager>()
            .get_mut("story.yaml")
            .unwrap()
            .run_chapter("Main")
            .expect("Could not run chapter");
    }
}
