use oxygengine::ha_renderer::mesh::transformers;
use oxygengine::prelude::*;

#[derive(Debug, Default)]
pub struct GameState {
    phase: f32,
}

impl State for GameState {
    fn on_enter(&mut self, universe: &mut Universe) {
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();
        let mut database = universe.expect_resource_mut::<AssetsDatabase>();
        database.defer_lately_cleanup();

        let deformer = Deformer::default().with_area(
            "test",
            DeformerArea {
                rectangle: rect(0.0, 0.0, 400.0, 400.0),
                cols: 1,
                rows: 1,
            },
        );
        let geometry = SurfaceGridFactory {
            cols: 10,
            rows: 10,
            cell_size: vec2(40.0, 40.0),
            ..Default::default()
        }
        .geometry(true)
        .unwrap();
        let geometry =
            transformers::fill_column::fill_column(geometry, "@deformer-area", "test", None)
                .unwrap();
        let geometry = transformers::apply_deformer::apply_deformer(geometry, &deformer).unwrap();
        // println!("{}", transformers::debug::debug(&geometry).unwrap());

        database.insert(Asset::new(
            "mesh",
            "@mesh/test",
            MeshAsset::Geometry(GeometryMeshAsset {
                vertex_data: MeshVertexData {
                    color: false,
                    texture: true,
                    skinning: false,
                    deforming: true,
                },
                factory: GeometryFactory(geometry),
            }),
        ));

        database.insert(Asset::new(
            "material",
            "@material/test",
            MaterialAsset::Graph {
                default_values: Default::default(),
                draw_options: MaterialDrawOptions::default(),
                content: material_graph! {
                    inputs {
                        [vertex] inout TextureCoord: vec3 = {vec3(0.0, 0.0, 0.0)};
                    }

                    outputs {
                        [fragment] inout BaseColor: vec4;
                    }

                    [(append_vec4,
                        a: (fract_vec3, v: (mul_vec3,
                            a: [TextureCoord => vColor],
                            b: (fill_vec3, v: {10.0})
                        )),
                        b: {1.0}
                    ) -> BaseColor]
                },
            },
        ));

        database.insert(Asset::new(
            "rig",
            "@rig/test",
            RigAsset::new(Default::default(), deformer, vec![]),
        ));

        commands.schedule(SpawnEntity::from_bundle((
            Name("main-camera".into()),
            HaCamera::default()
                .with_projection(HaCameraProjection::Orthographic(HaCameraOrthographic {
                    scaling: HaCameraOrtographicScaling::FitToView(1000.0.into(), false),
                    centered: true,
                    ignore_depth_planes: false,
                }))
                .with_pipeline(PipelineSource::Registry("default".to_owned())),
            HaDefaultCamera,
            HaTransform::translation(vec3(200.0, -280.0, 0.0)),
            NonPersistent,
        )));

        commands.schedule(SpawnEntity::from_bundle((
            Name("flag".into()),
            HaMeshInstance {
                reference: MeshReference::Asset("@mesh/test".to_owned()),
                ..Default::default()
            },
            HaMaterialInstance::new(MaterialReference::Asset("@material/test".to_owned())),
            {
                let mut instance = HaRigInstance::default();
                instance.set_asset("@rig/test");
                instance
            },
            HaTransform::translation(vec3(-200.0, -560.0, 0.0)),
            NonPersistent,
        )));

        commands.schedule(SpawnEntity::from_bundle((
            Name("watcher".into()),
            HaMeshInstance {
                reference: MeshReference::Asset("skeletons/watcher/mesh.json".to_owned()),
                ..Default::default()
            },
            HaMaterialInstance::new(MaterialReference::Asset(
                "@material/graph/surface/flat/texture-2d".to_owned(),
            ))
            .with_value(
                "mainImage",
                MaterialValue::sampler_2d(ImageReference::Asset(
                    "skeletons/watcher/image.json".to_owned(),
                )),
            ),
            {
                let mut instance = HaRigInstance::default();
                instance.set_asset("skeletons/watcher/rig.json");
                instance
                    .control
                    .property("animation-asset")
                    .set("skeletons/watcher/animation.json".to_owned());
                instance
            },
            HaTransform::default(),
            NonPersistent,
        )));
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let (world, life_cycle, input) =
            universe.query_resources::<(WorldRef, &AppLifeCycle, &mut InputController)>();

        let walk = input.trigger_or_default("walk").is_pressed();
        let run = input.trigger_or_default("run").is_pressed();
        let watch = input.trigger_or_default("watch").is_pressed();
        let play = input.trigger_or_default("play").is_pressed();

        for (_, instance) in world.query::<&mut HaRigInstance>().iter() {
            if play {
                instance
                    .control
                    .property("playing")
                    .or_default::<bool>()
                    .and_modify::<bool>(|playing| *playing = !*playing);
            }
            if walk {
                instance.control.property("state").set("#walk".to_owned());
            } else if run {
                instance.control.property("state").set("#run".to_owned());
            } else if watch {
                instance.control.property("state").set("#watch".to_owned());
            }
        }

        self.phase += life_cycle.delta_time_seconds() * 2.0;

        for (_, instance) in world.query::<&mut HaRigInstance>().iter() {
            if let Some(mut area) = instance.deformer.write_area("test") {
                if let Some(point) = area.write(0, 0) {
                    if let HaDeformerTangents::Mirrored {
                        horizontal,
                        vertical,
                    } = &mut point.tangents
                    {
                        vertical.x = self.phase.cos() * 100.0;
                        horizontal.y = self.phase.sin() * 200.0;
                    }
                }
                if let Some(point) = area.write(0, 1) {
                    if let HaDeformerTangents::Mirrored {
                        horizontal,
                        vertical,
                    } = &mut point.tangents
                    {
                        vertical.x = self.phase.cos() * 100.0;
                        horizontal.y = self.phase.sin() * 200.0;
                    }
                }
                if let Some(point) = area.write(1, 0) {
                    if let HaDeformerTangents::Mirrored {
                        horizontal,
                        vertical,
                    } = &mut point.tangents
                    {
                        vertical.x = self.phase.sin() * 100.0;
                        horizontal.y = self.phase.cos() * 200.0;
                    }
                }
                if let Some(point) = area.write(1, 1) {
                    if let HaDeformerTangents::Mirrored {
                        horizontal,
                        vertical,
                    } = &mut point.tangents
                    {
                        vertical.x = self.phase.sin() * 100.0;
                        horizontal.y = self.phase.cos() * 200.0;
                    }
                }
            }
        }

        StateChange::None
    }
}
