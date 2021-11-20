use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut prefabs = universe.expect_resource_mut::<PrefabManager>();
        prefabs.instantiate("Level_0x0", universe).unwrap();
        prefabs.instantiate("Level_0x1", universe).unwrap();
        prefabs.instantiate("Level_1x0", universe).unwrap();
        prefabs.instantiate("Level_1x1", universe).unwrap();
    }
}
