use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("scene", universe)
            .unwrap();
    }
}
