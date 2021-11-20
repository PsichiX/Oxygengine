use crate::story::Story;
use core::Scalar;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct VnStoryManager {
    stories: HashMap<String, Story>,
    registered: Vec<String>,
    unregistered: Vec<String>,
    lately_registered: Vec<String>,
    lately_unregistered: Vec<String>,
}

impl VnStoryManager {
    pub fn register_story(&mut self, name: &str, mut story: Story) {
        story.initialize();
        self.registered.push(name.to_owned());
        self.stories.insert(name.to_owned(), story);
    }

    pub fn unregister_story(&mut self, name: &str) -> Option<Story> {
        self.unregistered.push(name.to_owned());
        self.stories.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&Story> {
        self.stories.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Story> {
        self.stories.get_mut(name)
    }

    pub fn stories(&self) -> impl Iterator<Item = &Story> {
        self.stories.values()
    }

    pub fn stories_mut(&mut self) -> impl Iterator<Item = &mut Story> {
        self.stories.values_mut()
    }

    pub fn stories_names(&self) -> impl Iterator<Item = &str> {
        self.stories.keys().map(|id| id.as_str())
    }

    pub fn lately_registered(&self) -> impl Iterator<Item = &str> {
        self.lately_registered.iter().map(|id| id.as_str())
    }

    pub fn lately_unregistered(&self) -> impl Iterator<Item = &str> {
        self.lately_unregistered.iter().map(|id| id.as_str())
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.registered.is_empty() {
            self.lately_registered.clear();
        } else {
            self.lately_registered = std::mem::take(&mut self.registered);
        }
        if self.unregistered.is_empty() {
            self.lately_unregistered.clear();
        } else {
            self.lately_unregistered = std::mem::take(&mut self.unregistered);
        }
        for story in self.stories.values_mut() {
            drop(story.process(delta_time));
        }
    }
}
