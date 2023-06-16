use crate::{
    ecs::Entity,
    prefab::{Prefab, PrefabComponent, PrefabError, PrefabProxy},
    state::StateToken,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tag(pub Cow<'static, str>);

impl Prefab for Tag {}
impl PrefabComponent for Tag {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Name(pub Cow<'static, str>);

impl Prefab for Name {}
impl PrefabComponent for Name {}

#[derive(Debug, Default, Clone)]
pub struct NonPersistent(pub StateToken);

impl PrefabProxy<NonPersistentPrefabProxy> for NonPersistent {
    fn from_proxy_with_extras(
        _: NonPersistentPrefabProxy,
        _: &HashMap<String, Entity>,
        state_token: StateToken,
    ) -> Result<Self, PrefabError> {
        Ok(NonPersistent(state_token))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NonPersistentPrefabProxy;

impl Prefab for NonPersistentPrefabProxy {}

#[derive(Clone)]
pub struct Events<T>
where
    T: Send + Sync,
{
    buffer: VecDeque<T>,

    capacity: Option<usize>,

    pub auto_clear: bool,
}

impl<T> Default for Events<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self::new(None, true)
    }
}

impl<T> Events<T>
where
    T: Send + Sync,
{
    pub fn new(capacity: Option<usize>, auto_clear: bool) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity.unwrap_or_default()),
            capacity,
            auto_clear,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn consume(&mut self) -> impl Iterator<Item = T> + '_ {
        self.buffer.drain(..)
    }

    pub fn consume_if<F>(&mut self, mut f: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        if self.buffer.is_empty() {
            return Default::default();
        }
        let mut result = Vec::with_capacity(self.buffer.len());
        let mut buffer = VecDeque::with_capacity(self.buffer.capacity());
        for message in self.buffer.drain(..) {
            if f(&message) {
                result.push(message);
            } else {
                buffer.push_back(message);
            }
        }
        result
    }

    pub fn send(&mut self, message: T) {
        if let Some(capacity) = self.capacity {
            if self.buffer.len() >= capacity {
                self.buffer.pop_front();
            }
        }
        self.buffer.push_back(message);
    }

    pub fn try_send(&mut self, message: T) -> bool {
        if let Some(capacity) = self.capacity {
            if self.buffer.len() >= capacity {
                return false;
            }
        }
        self.buffer.push_back(message);
        true
    }
}

impl<T> PrefabProxy<EventsPrefabProxy<T>> for Events<T>
where
    T: Send + Sync + 'static,
{
    fn from_proxy_with_extras(
        proxy: EventsPrefabProxy<T>,
        _: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        Ok(Events::new(proxy.capacity, proxy.auto_clear))
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct EventsPrefabProxy<T>
where
    T: Send + Sync,
{
    #[serde(default)]
    pub capacity: Option<usize>,
    #[serde(default = "EventsPrefabProxy::<T>::default_auto_clear")]
    pub auto_clear: bool,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}

impl<T> EventsPrefabProxy<T>
where
    T: Send + Sync,
{
    fn default_auto_clear() -> bool {
        true
    }
}

impl<T> Prefab for EventsPrefabProxy<T> where T: Send + Sync {}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::Component;

    #[test]
    fn test_component() {
        fn foo<T: Component>() {
            println!("{} is Component", std::any::type_name::<T>());
        }

        foo::<Tag>();
        foo::<Name>();
        foo::<NonPersistent>();
        foo::<Events<()>>();
    }
}
