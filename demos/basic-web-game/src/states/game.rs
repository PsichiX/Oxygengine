use oxygengine::prelude::*;

pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<PrefabManager>()
            .instantiate_world("scene", world)
            .unwrap();
    }
}
