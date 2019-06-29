use oxygengine::prelude::*;

pub struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = ReadExpect<'s, AppLifeCycle>;

    fn run(&mut self, lifecycle: Self::SystemData) {
        info!("FPS: {:?}", 1.0 / lifecycle.delta_time_seconds());
    }
}
