use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        let assets = world.read_resource::<AssetsDatabase>();
        let asset = assets
            .asset_by_path("vn-story://story.yaml")
            .expect("trying to use not loaded VN story asset");
        let asset = asset
            .get::<VnStoryAsset>()
            .expect("trying to use non-vn-story asset");
        let mut story = asset.get().clone();
        story
            .run_chapter("Main")
            .expect("could not run main chapter");
        world
            .write_resource::<VnStoryManager>()
            .register_story("main", story);
    }
}
