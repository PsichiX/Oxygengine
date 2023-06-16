use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut prefabs = universe.expect_resource_mut::<PrefabManager>();
        prefabs.instantiate("Level", universe).unwrap();
    }
}
