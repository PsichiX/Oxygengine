use oxygengine_composite_renderer::{component::CompositeTransform, math::Vec2};
use oxygengine_core::{
    app::AppBuilder,
    ecs::{Component, Join, ReadStorage, System, VecStorage, Write, WriteStorage},
    hierarchy::Parent,
    prefab::{Prefab, PrefabComponent, PrefabManager},
};
use oxygengine_physics_2d::{component::RigidBody2d, resource::Physics2dWorld};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::*;
}

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>, _: ()) {
    builder.install_system(
        ApplyPhysics2dToCompositeTransformSystem,
        "apply-physics-2d-to-composite-transform-renderer",
        &[],
    );
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Physics2dSyncCompositeTransform;

impl Component for Physics2dSyncCompositeTransform {
    type Storage = VecStorage<Self>;
}

impl Prefab for Physics2dSyncCompositeTransform {}
impl PrefabComponent for Physics2dSyncCompositeTransform {}

#[derive(Debug, Default)]
pub struct ApplyPhysics2dToCompositeTransformSystem;

impl<'s> System<'s> for ApplyPhysics2dToCompositeTransformSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Write<'s, Physics2dWorld>>,
        ReadStorage<'s, RigidBody2d>,
        WriteStorage<'s, CompositeTransform>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Physics2dSyncCompositeTransform>,
    );

    fn run(&mut self, (world, bodies, mut transforms, parents, syncs): Self::SystemData) {
        if world.is_none() {
            return;
        }

        let world: &mut Physics2dWorld = &mut world.unwrap();

        for (body, transform, _, _) in (&bodies, &mut transforms, !&parents, &syncs).join() {
            if let Some(handle) = body.handle() {
                if let Some(body) = world.body(handle) {
                    let isometry = body.position();
                    let p = isometry.translation;
                    let r = isometry.rotation.angle();
                    let s = transform.get_scale();
                    transform.apply(Vec2::new(p.x, p.y), r, s);
                }
            }
        }
    }
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<Physics2dSyncCompositeTransform>(
        "Physics2dSyncCompositeTransform",
    );
}
