use crate::{
    ecs::Entity,
    prefab::{Prefab, PrefabComponent, PrefabError, PrefabProxy},
    state::StateToken,
};
use oxygengine_ignite_derive::Ignite;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tag(pub Cow<'static, str>);

impl Prefab for Tag {}
impl PrefabComponent for Tag {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Name(pub Cow<'static, str>);

impl Prefab for Name {}
impl PrefabComponent for Name {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
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
    }
}
