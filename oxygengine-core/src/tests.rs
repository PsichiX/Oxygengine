#![cfg(test)]
use super::app::{App, AppLifeCycle, AppRunner, SyncAppRunner};
use specs::prelude::*;

struct Counter {
    pub times: isize,
}

impl Component for Counter {
    type Storage = VecStorage<Self>;
}

struct CounterSystem;

impl<'s> System<'s> for CounterSystem {
    type SystemData = (Write<'s, AppLifeCycle>, WriteStorage<'s, Counter>);

    fn run(&mut self, (mut lifecycle, mut counters): Self::SystemData) {
        for counter in (&mut counters).join() {
            counter.times -= 1;
            println!("{:?}", counter.times);
            if counter.times <= 0 {
                lifecycle.running = false;
            }
        }
    }
}

#[test]
fn test_general() {
    let mut app = App::build()
        .with_system(CounterSystem, "counter", &[])
        .build();

    app.world_mut()
        .create_entity()
        .with(Counter { times: 10 })
        .build();

    let mut runner = AppRunner::new(app);
    drop(runner.run::<SyncAppRunner, ()>());
}
