#![cfg(test)]
use super::{
    app::{App, AppLifeCycle, AppRunner, StandardAppTimer, SyncAppRunner},
    hierarchy::{hierarchy_find, Name, Parent},
    state::{State, StateChange},
};
use specs::prelude::*;

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
