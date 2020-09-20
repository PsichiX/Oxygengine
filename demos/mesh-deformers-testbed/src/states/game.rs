use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    logo: Option<Entity>,
    phase: Scalar,
}

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        let logo = world
            .write_resource::<PrefabManager>()
            .instantiate_world("scene", world)
            .unwrap()[1];
        self.logo = Some(logo);
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let dt = world.read_resource::<AppLifeCycle>().delta_time_seconds();
        self.phase += dt;
        if let Some(logo) = self.logo {
            if let Some(mesh) = world.write_component::<CompositeMesh>().get_mut(logo) {
                mesh.with_bone_local_transform("", |t| t.set_rotation(self.phase.sin() * 0.05));
                mesh.with_bone_local_transform("a", |t| t.set_rotation(self.phase.sin() * 0.1));
                mesh.with_bone_local_transform("b", |t| t.set_rotation(self.phase.sin() * 0.15));
                mesh.with_bone_local_transform("c", |t| t.set_rotation(self.phase.sin() * 0.2));
            }
        }
        StateChange::None
    }
}
