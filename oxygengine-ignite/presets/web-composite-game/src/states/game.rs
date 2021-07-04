use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    camera: Option<Entity>,
    camera_ui: Option<Entity>,
}

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        // instantiate world objects from scene prefab.
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("scene", universe)
            .unwrap();
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        if let Some(camera) = self.camera {
            // check if we pressed left mouse button.
            let input = universe.expect_resource::<InputController>();
            if input.trigger_or_default("pointer-action").is_pressed() {
                // get mouse screen space coords.
                let x = input.axis_or_default("pointer-x");
                let y = input.axis_or_default("pointer-y");
                let point = [x, y].into();

                // convert mouse coords from screen space to world space.
                if let Some(pos) = universe
                    .expect_resource::<CompositeCameraCache>()
                    .screen_to_world_space(camera, point)
                {
                    // instantiate object from prefab and store its entity.
                    let instance = universe
                        .expect_resource_mut::<PrefabManager>()
                        .instantiate("instance", universe)
                        .unwrap()[0];
                    universe
                        .world()
                        .query_one::<&mut CompositeTransform>(instance)
                        .unwrap()
                        .get()
                        .unwrap()
                        .set_translation(pos);
                }
            }
        } else {
            // find and store camera entity by its name.
            self.camera = universe
                .expect_resource::<Hierarchy>()
                .entity_by_name("camera");
        }

        StateChange::None
    }
}
