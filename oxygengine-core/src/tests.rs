#![cfg(test)]
use super::{
    app::{App, AppLifeCycle, AppRunner, StandardAppTimer, SyncAppRunner},
    state::State,
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
            if counter.times <= 0 {
                lifecycle.running = false;
            }
        }
    }
}

struct Example;

impl State for Example {}

#[test]
fn test_general() {
    let mut app = App::build()
        .with_system(CounterSystem, "counter", &[])
        .build(Example, StandardAppTimer::default());

    app.world_mut()
        .create_entity()
        .with(Counter { times: 10 })
        .build();

    let mut runner = AppRunner::new(app);
    drop(runner.run::<SyncAppRunner, ()>());
}
