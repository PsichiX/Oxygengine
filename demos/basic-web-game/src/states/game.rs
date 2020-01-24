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

        // find and store camera entity by its name.
        self.camera = entity_find_world("camera", world);
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let Some(camera) = self.camera {
            // check if we pressed left mouse button.
            let input = &world.read_resource::<InputController>();
            if input.trigger_or_default("mouse-left").is_pressed() {
                // get mouse unit space coords.
                let x = input.axis_or_default("mouse-x");
                let y = input.axis_or_default("mouse-y");
                // convert mouse coords from unit space to world space.
                if let Some(pos) = world
                    .read_resource::<CompositeCameraCache>()
                    .transform_point(camera, [x, y].into())
                {
                    // instantiate object from prefab and store its entity.
                    let instance = world
                        .write_resource::<PrefabManager>()
                        .instantiate_world("instance", world)
                        .unwrap()[0];
                    // get instance transform and set its position from mouse world space coords.
                    if let Some(transform) = world
                        .write_storage::<CompositeTransform>()
                        .get_mut(instance)
                    {
                        transform.set_translation(pos)
                    }
                }
            }
        }
        StateChange::None
    }
}
