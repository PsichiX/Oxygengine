use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    camera: Option<Entity>,
    player: Option<Entity>,
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // instantiate world objects from scene prefab.
        let camera = world
            .write_resource::<PrefabManager>()
            .instantiate_world("new-bark-town", world)
            .unwrap()[0];
        self.camera = Some(camera);

        // instantiate player from prefab.
        let player = world
            .write_resource::<PrefabManager>()
            .instantiate_world("player", world)
            .unwrap()[0];
        self.player = Some(player);

        // setup created player instance.
        world.read_resource::<LazyUpdate>().exec(move |world| {
            let mut transform = <CompositeTransform>::fetch(world, player);
            let pos = Vec2::new(16.0 * 12.0, 16.0 * 11.0);
            transform.set_translation(pos);
        });
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let (Some(player), Some(camera)) = (self.player, self.camera) {
            let mut transforms = world.write_storage::<CompositeTransform>();
            // NOTE: REMEMBER THAT PREFABS ARE INSTANTIATED IN NEXT FRAME, SO THEY MIGHT NOT EXIST
            // AT FIRST SO HANDLE THAT.
            if let Some(player_transform) = transforms.get(player) {
                let player_position = player_transform.get_translation();
                if let Some(camera_transform) = transforms.get_mut(camera) {
                    camera_transform.set_translation(player_position);
                }
            }
        }

        StateChange::None
    }
}
