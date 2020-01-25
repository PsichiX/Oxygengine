use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    camera: Option<Entity>,
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // instantiate world objects from scene prefab.
        world
            .write_resource::<PrefabManager>()
            .instantiate_world("scene", world)
            .unwrap();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let Some(camera) = self.camera {
            // check if we pressed left mouse button.
            let input = &world.read_resource::<InputController>();
            if input.trigger_or_default("mouse-left").is_pressed() {
                // get mouse screen space coords.
                let x = input.axis_or_default("mouse-x");
                let y = input.axis_or_default("mouse-y");
                // convert mouse coords from screen space to world space.
                if let Some(pos) = world
                    .read_resource::<CompositeCameraCache>()
                    .screen_to_world_space(camera, [x, y].into())
                {
                    // instantiate object from prefab and store its entity.
                    let instance = world
                        .write_resource::<PrefabManager>()
                        .instantiate_world("instance", world)
                        .unwrap()[0];
                    // LazyUpdate::exec() runs code after all systems are done, so it's perfect to
                    // modify components of entities created from prefab there.
                    // note this `move` within closure definition - since we ue `pos` and `instance`
                    // objects from outside of closure scope, rust has to be informed that we want
                    // to move ownership of that objects to inside of closure scope.
                    world.read_resource::<LazyUpdate>().exec(move |world| {
                        // fetch CompositeTransform from instance and set its position.
                        // note that we can fetch multiple components at once if we pack them in
                        // tuple (up to 26 components) just like that:
                        // ```
                        // let (mut t, s) = <(CompositeTransform, Speed)>::fetch(world, instance);
                        // let pos = t.get_translation() + t.get_direction() * s.0;
                        // t.set_translation(pos);
                        // ```
                        let mut transform = <CompositeTransform>::fetch(world, instance);
                        transform.set_translation(pos);
                    });
                }
            }
        } else {
            // find and store camera entity by its name.
            self.camera = entity_find_world("camera", world);
        }
        StateChange::None
    }
}
