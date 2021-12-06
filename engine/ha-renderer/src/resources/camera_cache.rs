use crate::components::camera::HaStageCameraInfo;
use core::ecs::Entity;
use std::any::TypeId;

#[derive(Debug, Default, Clone)]
pub struct CameraCache {
    pub(crate) default_entity: Option<Entity>,
    pub(crate) info: Vec<(Entity, TypeId, Option<String>, HaStageCameraInfo)>,
}

impl CameraCache {
    pub fn default_entity(&self) -> Option<Entity> {
        self.default_entity
    }

    pub fn get_all<T: 'static>(&self, entity: Entity) -> impl Iterator<Item = &HaStageCameraInfo> {
        let type_id = TypeId::of::<T>();
        self.info
            .iter()
            .filter(move |(ent, tid, _, _)| ent == &entity && tid == &type_id)
            .map(|(_, _, _, info)| info)
    }

    pub fn get<T: 'static>(&self, entity: Entity, nth: usize) -> Option<&HaStageCameraInfo> {
        self.get_all::<T>(entity).nth(nth)
    }

    pub fn get_first<T: 'static>(&self, entity: Entity) -> Option<&HaStageCameraInfo> {
        self.get_all::<T>(entity).next()
    }

    pub fn named_get_all<'a, T: 'static>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a HaStageCameraInfo> {
        let type_id = TypeId::of::<T>();
        self.info
            .iter()
            .filter(move |(_, tid, n, _)| {
                n.as_ref().map(|n| n == name).unwrap_or_default() && tid == &type_id
            })
            .map(|(_, _, _, info)| info)
    }

    pub fn named_get<'a, T: 'static>(
        &'a self,
        name: &'a str,
        nth: usize,
    ) -> Option<&'a HaStageCameraInfo> {
        self.named_get_all::<T>(name).nth(nth)
    }

    pub fn named_get_first<'a, T: 'static>(
        &'a self,
        name: &'a str,
    ) -> Option<&'a HaStageCameraInfo> {
        self.named_get_all::<T>(name).next()
    }

    pub fn default_get_all<T: 'static>(&self) -> Option<impl Iterator<Item = &HaStageCameraInfo>> {
        Some(self.get_all::<T>(self.default_entity?))
    }

    pub fn default_get<T: 'static>(&self, nth: usize) -> Option<&HaStageCameraInfo> {
        self.default_get_all::<T>()
            .and_then(|mut iter| iter.nth(nth))
    }

    pub fn default_get_first<T: 'static>(&self) -> Option<&HaStageCameraInfo> {
        self.default_get_all::<T>().and_then(|mut iter| iter.next())
    }
}
