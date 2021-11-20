use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("scene", universe)
            .unwrap();
        universe
            .expect_resource_mut::<VnStoryManager>()
            .get_mut("vn/story.yaml")
            .unwrap()
            .run_chapter("Main")
            .expect("Could not run chapter");
    }
}
