#![cfg(test)]

use crate::{
    app::{App, AppRunner, StandardAppTimer, SyncAppRunner},
    assets::{database::AssetsDatabase, protocols::prefab::PrefabAsset},
    ecs::{
        commands::{DespawnEntity, SpawnEntity, UniverseCommand},
        components::Name,
        hierarchy::{Hierarchy, Parent},
        life_cycle::EntityChanges,
        pipeline::{engines::sequence::SequencePipelineEngine, LinearPipelineBuilder},
        Bundle, Entity, Universe,
    },
    fetch::engines::map::MapFetchEngine,
    localization::Localization,
    log::{logger_setup, DefaultLogger},
    prefab::{
        Prefab, PrefabManager, PrefabScene, PrefabSceneEntity, PrefabSceneEntityData, PrefabValue,
    },
    state::{State, StateChange},
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct ExamplePrefab(bool);

impl State for ExamplePrefab {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .resource_mut::<AssetsDatabase>()
            .unwrap()
            .load("prefab://scene.yaml")
            .unwrap();
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        if self.0 {
            StateChange::Pop
        } else {
            let (assets, mut prefabs) =
                universe.query_resources::<(&AssetsDatabase, &mut PrefabManager)>();
            if let Some(asset) = assets.asset_by_path("prefab://scene.yaml") {
                self.0 = true;
                let prefab = asset
                    .get::<PrefabAsset>()
                    .expect("scene.ron is not a prefab asset")
                    .get();
                let entities = prefabs.load_scene_from_prefab(prefab, universe).unwrap();
                println!("scene.yaml asset finally loaded: {:?}", entities);
            } else {
                println!("scene.yaml asset not loaded yet");
            }
            StateChange::None
        }
    }
}

#[derive(Bundle)]
struct Root {
    pub name: Name,
}

#[derive(Bundle)]
struct Child {
    pub name: Name,
    pub parent: Parent,
}

#[test]
fn test_prefabs() {
    let prefab = PrefabScene {
        template_name: None,
        dependencies: vec![],
        entities: vec![
            PrefabSceneEntity::Data(PrefabSceneEntityData {
                uid: None,
                components: {
                    let mut map = HashMap::new();
                    map.insert(
                        "Name".to_owned(),
                        PrefabValue::String("some name".to_owned()),
                    );
                    map
                },
            }),
            PrefabSceneEntity::Template("some template".to_owned()),
        ],
    };
    println!("Prefab string:\n{}", prefab.to_prefab_string().unwrap());

    let mut files = HashMap::new();
    files.insert(
        "scene.yaml".to_owned(),
        br#"
        entities:
          - Data:
              components:
                Name: hello
                Tag: greting
                NonPersistent:
        "#
        .to_vec(),
    );
    let app = App::build::<LinearPipelineBuilder>()
        .with_bundle(
            crate::assets::bundle_installer,
            (MapFetchEngine::new(files), |_| {}),
        )
        .unwrap()
        .with_bundle(crate::prefab::bundle_installer, |_| {})
        .unwrap()
        .build::<SequencePipelineEngine, _, _>(
            ExamplePrefab::default(),
            StandardAppTimer::default(),
        );

    let _ = AppRunner::new(app).run(SyncAppRunner::default());
}

#[test]
fn test_hierarchy_find() {
    let mut app = App::build::<LinearPipelineBuilder>()
        .build_empty::<SequencePipelineEngine, _>(StandardAppTimer::default());
    let (root, child_a, child_b, child_c) = {
        let mut world = app.multiverse.default_universe_mut().unwrap().world_mut();
        let root = world.spawn(Root {
            name: Name("root".into()),
        });
        let child_a = world.spawn(Child {
            name: Name("a".into()),
            parent: Parent(root),
        });
        let child_b = world.spawn(Child {
            name: Name("b".into()),
            parent: Parent(child_a),
        });
        let child_c = world.spawn(Child {
            name: Name("c".into()),
            parent: Parent(root),
        });
        (root, child_a, child_b, child_c)
    };
    app.process();
    let hierarchy = app
        .multiverse
        .default_universe()
        .unwrap()
        .expect_resource::<Hierarchy>();
    assert_eq!(hierarchy.find(None, "root"), Some(root));
    assert_eq!(hierarchy.find(Some(root), ""), Some(root));
    assert_eq!(hierarchy.find(Some(root), "."), Some(root));
    assert_eq!(hierarchy.find(Some(root), ".."), None);
    assert_eq!(hierarchy.find(Some(root), "a"), Some(child_a));
    assert_eq!(hierarchy.find(Some(root), "a/"), Some(child_a));
    assert_eq!(hierarchy.find(Some(root), "a/."), Some(child_a));
    assert_eq!(hierarchy.find(Some(root), "a/.."), Some(root));
    assert_eq!(hierarchy.find(Some(root), "a/../.."), None);
    assert_eq!(hierarchy.find(None, "b"), Some(child_b));
    assert_eq!(hierarchy.find(None, "b/.."), Some(child_a));
    assert_eq!(hierarchy.find(None, "b/../.."), Some(root));
    assert_eq!(hierarchy.find(None, "c"), Some(child_c));
    assert_eq!(hierarchy.find(None, "c/"), Some(child_c));
    assert_eq!(hierarchy.find(None, "c/."), Some(child_c));
    assert_eq!(hierarchy.find(None, "c/.."), Some(root));
    assert_eq!(hierarchy.find(None, "c/../.."), None);
    assert_eq!(hierarchy.find(None, "a/b"), Some(child_b));
    assert_eq!(hierarchy.find(None, "a/b/"), Some(child_b));
    assert_eq!(hierarchy.find(None, "a/b/.."), Some(child_a));
    assert_eq!(hierarchy.find(None, "a/b/../.."), Some(root));
    assert_eq!(hierarchy.find(None, "a/b/../../.."), None);
    assert_eq!(hierarchy.find(None, "a/b/../../c"), Some(child_c));
}

#[test]
fn test_entity_life_cycle() {
    fn set(container: &[Entity]) -> HashSet<Entity> {
        container.iter().copied().collect()
    }

    let mut app = App::build::<LinearPipelineBuilder>()
        .build_empty::<SequencePipelineEngine, _>(StandardAppTimer::default());
    {
        let changes = app
            .multiverse
            .default_universe()
            .unwrap()
            .expect_resource::<EntityChanges>();
        assert_eq!(&changes.spawned().collect::<Vec<_>>(), &[]);
        assert_eq!(&changes.despawned().collect::<Vec<_>>(), &[]);
    }
    let (root, e1) = {
        let universe = app.multiverse.default_universe_mut().unwrap();
        universe
            .expect_resource_mut::<EntityChanges>()
            .skip_clearing = true;
        let root = SpawnEntity::from_bundle(Root {
            name: Name("root".into()),
        })
        .execute(universe);
        let e1 = SpawnEntity::from_bundle(Child {
            name: Name("a".into()),
            parent: Parent(root),
        })
        .execute(universe);
        (root, e1)
    };
    app.process();
    {
        let changes = app
            .multiverse
            .default_universe()
            .unwrap()
            .expect_resource::<EntityChanges>();
        assert_eq!(changes.spawned().collect::<HashSet<_>>(), set(&[root, e1]));
        assert_eq!(changes.despawned().collect::<HashSet<_>>(), set(&[]));
        let world = app.multiverse.default_universe_mut().unwrap().world_mut();
        assert_eq!(
            world.iter().map(|id| id.entity()).collect::<HashSet<_>>(),
            set(&[root, e1])
        );
    }
    app.process();
    {
        let changes = app
            .multiverse
            .default_universe()
            .unwrap()
            .expect_resource::<EntityChanges>();
        assert_eq!(&changes.spawned().collect::<Vec<_>>(), &[]);
        assert_eq!(&changes.despawned().collect::<Vec<_>>(), &[]);
        let world = app.multiverse.default_universe_mut().unwrap().world_mut();
        assert_eq!(
            world.iter().map(|id| id.entity()).collect::<HashSet<_>>(),
            set(&[root, e1])
        );
    }
    {
        let universe = app.multiverse.default_universe_mut().unwrap();
        universe
            .expect_resource_mut::<EntityChanges>()
            .skip_clearing = true;
        DespawnEntity(root).run(universe);
        assert_eq!(
            universe
                .world()
                .iter()
                .map(|id| id.entity())
                .collect::<HashSet<_>>(),
            set(&[e1])
        );
    }
    app.process();
    {
        let changes = app
            .multiverse
            .default_universe()
            .unwrap()
            .expect_resource::<EntityChanges>();
        assert_eq!(changes.spawned().collect::<HashSet<_>>(), set(&[]));
        assert_eq!(
            changes.despawned().collect::<HashSet<_>>(),
            set(&[root, e1])
        );
        let world = app.multiverse.default_universe_mut().unwrap().world_mut();
        assert_eq!(
            world.iter().map(|id| id.entity()).collect::<HashSet<_>>(),
            set(&[])
        );
    }
    app.process();
    {
        let changes = app
            .multiverse
            .default_universe()
            .unwrap()
            .expect_resource::<EntityChanges>();
        assert_eq!(changes.spawned().collect::<HashSet<_>>(), set(&[]));
        assert_eq!(changes.despawned().collect::<HashSet<_>>(), set(&[]));
        let world = app.multiverse.default_universe_mut().unwrap().world_mut();
        assert_eq!(
            world.iter().map(|id| id.entity()).collect::<HashSet<_>>(),
            set(&[])
        );
    }
}

#[test]
fn test_logger() {
    logger_setup(DefaultLogger);
    info!("my logger {}", "info");
    warn!("my logger {}", "warn");
    error!("my logger {}", "error");
}

#[test]
fn test_localization() {
    let mut loc = Localization::default();
    loc.add_text(
        "hello",
        "lang",
        "Hello |@name|, you've got |@score| points! \\| |@bye",
    );
    loc.set_current_language(Some("lang".to_owned()));
    let text = localization_format_text!(loc, "hello", name => "Person", score => 42).unwrap();
    assert_eq!(text, "Hello Person, you've got 42 points! | {@bye}");
}
