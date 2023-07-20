use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut prefabs = universe.expect_resource_mut::<PrefabManager>();
        // TODO: for now we load all world chunks, later we will stream them with visibility volumes.
        prefabs.instantiate("Level_0x0", universe).unwrap();
        prefabs.instantiate("Level_0x1", universe).unwrap();
        prefabs.instantiate("Level_1x0", universe).unwrap();
        prefabs.instantiate("Level_1x1", universe).unwrap();
    }
}
