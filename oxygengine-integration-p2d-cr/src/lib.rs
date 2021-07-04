use oxygengine_composite_renderer::{component::CompositeTransform, math::Vec2};
use oxygengine_core::{
    app::AppBuilder,
    ecs::{
        hierarchy::Parent,
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Comp, Universe, WorldRef,
    },
    prefab::{Prefab, PrefabComponent, PrefabManager},
    Ignite,
};
use oxygengine_physics_2d::{component::RigidBody2d, resource::Physics2dWorld};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::*;
}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Physics2dSyncCompositeTransform;

impl Prefab for Physics2dSyncCompositeTransform {}
impl PrefabComponent for Physics2dSyncCompositeTransform {}

pub type ApplyPhysics2dToCompositeTransformSystemResources<'a> = (
    WorldRef,
    &'a Physics2dWorld,
    Comp<&'a RigidBody2d>,
    Comp<&'a mut CompositeTransform>,
    Comp<&'a Parent>,
    Comp<&'a Physics2dSyncCompositeTransform>,
);

pub fn apply_physics_2d_to_composite_transform_system(universe: &mut Universe) {
    let (world, physics, ..) =
        universe.query_resources::<ApplyPhysics2dToCompositeTransformSystemResources>();

    for (_, (body, transform)) in world
        .query::<(&RigidBody2d, &mut CompositeTransform)>()
        .without::<Parent>()
        .with::<Physics2dSyncCompositeTransform>()
        .iter()
    {
        if let Some(handle) = body.handle() {
            if let Some(body) = physics.body(handle) {
                let isometry = body.position();
                let p = isometry.translation;
                let r = isometry.rotation.angle();
                let s = transform.get_scale();
                transform.apply(Vec2::new(p.x, p.y), r, s);
            }
        }
    }
}

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_system::<ApplyPhysics2dToCompositeTransformSystemResources>(
        "apply-physics-2d-to-composite-transform",
        apply_physics_2d_to_composite_transform_system,
        &[],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<Physics2dSyncCompositeTransform>(
        "Physics2dSyncCompositeTransform",
    );
}
