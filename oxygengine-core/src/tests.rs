#![cfg(test)]

use super::{
    app::{App, AppLifeCycle, AppRunner, StandardAppTimer, SyncAppRunner},
    assets::{database::AssetsDatabase, protocols::prefab::PrefabAsset},
    fetch::engines::map::MapFetchEngine,
    hierarchy::{hierarchy_find, HierarchyChangeRes, Name, Parent},
    log::{logger_setup, DefaultLogger},
    prefab::*,
    state::{State, StateChange},
};
use specs::prelude::*;
use std::collections::HashMap;

struct Counter {
    pub times: isize,
}

impl Component for Counter {
    type Storage = VecStorage<Self>;
}

struct CounterSystem;

impl<'s> System<'s> for CounterSystem {
    type SystemData = (WriteExpect<'s, AppLifeCycle>, WriteStorage<'s, Counter>);

    fn run(&mut self, (mut lifecycle, mut counters): Self::SystemData) {
        for counter in (&mut counters).join() {
            counter.times -= 1;
            println!("counter: {:?}", counter.times);
            if counter.times <= 0 {
                lifecycle.running = false;
            }
        }
    }
}

struct PrintableSystem;

impl<'s> System<'s> for PrintableSystem {
    type SystemData = ReadStorage<'s, Name>;

    fn run(&mut self, names: Self::SystemData) {
        for name in (&names).join() {
            println!("name: {:?}", name.0);
        }
    }
}

#[derive(Default)]
struct Example {
    root: Option<Entity>,
}

impl State for Example {
    fn on_enter(&mut self, world: &mut World) {
        world.create_entity().with(Counter { times: 10 }).build();

        let root = world.create_entity().with(Name("root".into())).build();

        world
            .create_entity()
            .with(Parent(root))
            .with(Name("child".into()))
            .build();

        self.root = Some(root);
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        if let Some(root) = self.root {
            world.delete_entity(root).unwrap();
            self.root = None;
        }
        StateChange::None
    }
}

#[test]
fn test_general() {
    let app = App::build()
        .with_system(CounterSystem, "counter", &[])
        .with_system(PrintableSystem, "names", &[])
        .build(Example::default(), StandardAppTimer::default());

    let mut runner = AppRunner::new(app);
    drop(runner.run(SyncAppRunner::default()));
}

#[derive(Default)]
struct ExamplePrefab {
    phase: usize,
}

impl State for ExamplePrefab {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<AssetsDatabase>()
            .load("prefab://scene.yaml")
            .unwrap();
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        match self.phase {
            0 => {
                if let Some(asset) = world
                    .read_resource::<AssetsDatabase>()
                    .asset_by_path("prefab://scene.yaml")
                {
                    let prefab = asset
                        .get::<PrefabAsset>()
                        .expect("scene.ron is not a prefab asset")
                        .get();
                    let entities = world
                        .write_resource::<PrefabManager>()
                        .load_scene_from_prefab_world(prefab, world)
                        .unwrap();
                    self.phase = 1;
                    println!("scene.yaml asset finally loaded: {:?}", entities);
                } else {
                    println!("scene.yaml asset not loaded yet");
                }
                StateChange::None
            }
            _ => StateChange::Pop,
        }
    }
}

#[test]
fn test_prefabs() {
    let prefab = PrefabScene {
        autoload: false,
        template_name: None,
        dependencies: vec![],
        entities: vec![
            PrefabSceneEntity::Data(PrefabSceneEntityData {
                uid: Some("uidname".to_owned()),
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
    let app = App::build()
        .with_bundle(
            crate::assets::bundle_installer,
            (MapFetchEngine::new(files), |_| {}),
        )
        .with_bundle(crate::prefab::bundle_installer, |_| {})
        .build(ExamplePrefab::default(), StandardAppTimer::default());

    let mut runner = AppRunner::new(app);
    drop(runner.run(SyncAppRunner::default()));
}

#[test]
fn test_hierarchy_find() {
    let mut app = App::build().build_empty(StandardAppTimer::default());
    let root = app
        .world_mut()
        .create_entity()
        .with(Name("root".into()))
        .build();
    let child_a = app
        .world_mut()
        .create_entity()
        .with(Name("a".into()))
        .with(Parent(root))
        .build();
    let child_b = app
        .world_mut()
        .create_entity()
        .with(Name("b".into()))
        .with(Parent(child_a))
        .build();
    let child_c = app
        .world_mut()
        .create_entity()
        .with(Name("c".into()))
        .with(Parent(root))
        .build();
    app.process();
    assert_eq!(hierarchy_find(root, "", app.world()), Some(root));
    assert_eq!(hierarchy_find(root, ".", app.world()), Some(root));
    assert_eq!(hierarchy_find(root, "..", app.world()), None);
    assert_eq!(hierarchy_find(root, "a", app.world()), Some(child_a));
    assert_eq!(hierarchy_find(root, "a/", app.world()), Some(child_a));
    assert_eq!(hierarchy_find(root, "a/.", app.world()), Some(child_a));
    assert_eq!(hierarchy_find(root, "a/..", app.world()), Some(root));
    assert_eq!(hierarchy_find(root, "a/../..", app.world()), None);
    assert_eq!(hierarchy_find(root, "b", app.world()), None);
    assert_eq!(hierarchy_find(root, "c", app.world()), Some(child_c));
    assert_eq!(hierarchy_find(root, "c/", app.world()), Some(child_c));
    assert_eq!(hierarchy_find(root, "c/.", app.world()), Some(child_c));
    assert_eq!(hierarchy_find(root, "c/..", app.world()), Some(root));
    assert_eq!(hierarchy_find(root, "c/../..", app.world()), None);
    assert_eq!(hierarchy_find(root, "a/b", app.world()), Some(child_b));
    assert_eq!(hierarchy_find(root, "a/b/", app.world()), Some(child_b));
    assert_eq!(hierarchy_find(root, "a/b/", app.world()), Some(child_b));
    assert_eq!(hierarchy_find(root, "a/b/..", app.world()), Some(child_a));
    assert_eq!(hierarchy_find(root, "a/b/../..", app.world()), Some(root));
    assert_eq!(hierarchy_find(root, "a/b/../../..", app.world()), None);
    assert_eq!(hierarchy_find(root, "a/b/../../..", app.world()), None);
}

#[test]
fn test_hierarchy_add_remove() {
    fn sorted(container: &[Entity]) -> Vec<Entity> {
        let mut result = container.to_vec();
        result.sort();
        result
    }

    let mut app = App::build().build_empty(StandardAppTimer::default());
    {
        let changes = app.world().read_resource::<HierarchyChangeRes>();
        assert_eq!(changes.added(), &[]);
        assert_eq!(changes.removed(), &[]);
    }
    let root = app
        .world_mut()
        .create_entity()
        .with(Name("root".into()))
        .build();
    let e1 = app.world_mut().create_entity().with(Parent(root)).build();
    app.process();
    {
        let changes = app.world().read_resource::<HierarchyChangeRes>();
        assert_eq!(sorted(changes.added()), sorted(&[root, e1]));
        assert_eq!(changes.removed(), &[]);
    }
    app.process();
    {
        let changes = app.world().read_resource::<HierarchyChangeRes>();
        assert_eq!(changes.added(), &[]);
        assert_eq!(changes.removed(), &[]);
    }
    app.world_mut().delete_entity(root).unwrap();
    app.world_mut().delete_entity(e1).unwrap();
    app.process();
    {
        let changes = app.world().read_resource::<HierarchyChangeRes>();
        assert_eq!(changes.added(), &[]);
        assert_eq!(sorted(changes.removed()), sorted(&[root, e1]));
    }
}

#[test]
fn test_logger() {
    logger_setup(DefaultLogger);
    info!("my logger {}", "info");
    warn!("my logger {}", "warn");
    error!("my logger {}", "error");
}
