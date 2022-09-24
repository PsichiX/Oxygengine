use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();

        commands.schedule(SpawnEntity::from_bundle((
            Name("main-camera".into()),
            HaCamera::default()
                .with_projection(HaCameraProjection::Orthographic(HaCameraOrthographic {
                    scaling: HaCameraOrtographicScaling::FitToView(1024.0.into(), false),
                    centered: true,
                    ignore_depth_planes: false,
                }))
                .with_viewport(RenderTargetViewport::Full)
                .with_pipeline(PipelineSource::Registry("default".to_owned())),
            HaDefaultCamera,
            HaTransform::translation(vec3(200.0, -280.0, 0.0)),
            NonPersistent,
        )));

        commands.schedule(SpawnEntity::from_bundle((
            Name("watcher".into()),
            HaMeshInstance {
                reference: MeshReference::Asset("skeletons/watcher/mesh.yaml".to_owned()),
                ..Default::default()
            },
            HaMaterialInstance::new(MaterialReference::Asset(
                "@material/graph/surface/flat/texture-2d".to_owned(),
            ))
            .with_value(
                "mainImage",
                MaterialValue::sampler_2d(ImageReference::Asset(
                    "skeletons/watcher/image.yaml".to_owned(),
                )),
            ),
            {
                let mut instance = HaSkeletonInstance::default();
                instance.set_skeleton("skeletons/watcher/skeleton.yaml");
                instance
            },
            {
                let mut instance = HaSkeletalAnimationInstance::default();
                instance.set_animation("skeletons/watcher/animation.yaml");
                instance.play("#walk");
                instance
            },
            HaTransform::default(),
            NonPersistent,
        )));
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let (world, input) = universe.query_resources::<(WorldRef, &mut InputController)>();

        let walk = input.trigger_or_default("walk").is_pressed();
        let run = input.trigger_or_default("run").is_pressed();
        let watch = input.trigger_or_default("watch").is_pressed();

        for (_, instance) in world.query::<&mut HaSkeletalAnimationInstance>().iter() {
            if walk {
                instance.play("#walk");
            } else if run {
                instance.play("#run");
            } else if watch {
                instance.play("#watch");
            }
        }

        StateChange::None
    }
}
