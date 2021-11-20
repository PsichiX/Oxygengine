#![cfg(test)]

use crate::{core::prefab::Prefab, story::Story};

#[test]
fn test_hello_story() {
    let content =
        std::fs::read_to_string("scripts/hello_world.yaml").expect("Could not read story");
    let mut story = Story::from_prefab_str(&content).expect("Could not deserialize story");
    story
        .run_chapter("Main")
        .expect("Could not run main chapter");
    let mut step = 0;
    while !story.is_complete() {
        story.process(1.0).expect("Error during story processing");
        step += 1;
        if step == 2 {
            story
                .select_dialogue_option(0)
                .expect("Could not select dialogue option");
        }
    }
    assert_eq!(step, 4);
}
