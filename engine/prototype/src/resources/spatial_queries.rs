use oxygengine_core::{ecs::Entity, Scalar};
use oxygengine_ha_renderer::math::{Rect, Vec2};
use rstar::{
    primitives::{GeomWithData, Rectangle},
    RTree, AABB,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct SpatialQueries {
    rtree: RTree<GeomWithData<Rectangle<[Scalar; 2]>, Entity>>,
    collisions: HashMap<Entity, HashSet<Entity>>,
}

impl SpatialQueries {
    pub fn clear(&mut self) {
        self.rtree = Default::default();
        self.collisions.clear();
    }

    pub fn add(&mut self, entity: Entity, rect: Rect) {
        self.rtree.insert(GeomWithData::new(
            Rectangle::from_corners([rect.x, rect.y], [rect.x + rect.w, rect.y + rect.h]),
            entity,
        ));
        for geom in self
            .rtree
            .locate_in_envelope_intersecting(&AABB::from_corners(
                [rect.x, rect.y],
                [rect.x + rect.w, rect.y + rect.h],
            ))
        {
            if geom.data != entity {
                self.collisions.entry(entity).or_default().insert(geom.data);
                self.collisions.entry(geom.data).or_default().insert(entity);
            }
        }
    }

    pub fn nearest(&self, point: Vec2) -> impl Iterator<Item = (Entity, Rect, Scalar)> + '_ {
        self.rtree
            .nearest_neighbor_iter_with_distance_2(&[point.x, point.y])
            .map(|(item, distance2)| {
                let lower = item.geom().lower();
                let upper = item.geom().upper();
                (
                    item.data,
                    Rect::new(lower[0], lower[1], upper[0] - lower[0], upper[1] - lower[1]),
                    distance2,
                )
            })
    }

    pub fn contains(&self, point: Vec2) -> impl Iterator<Item = (Entity, Rect)> + '_ {
        self.rtree
            .locate_all_at_point(&[point.x, point.y])
            .map(|item| {
                let lower = item.geom().lower();
                let upper = item.geom().upper();
                (
                    item.data,
                    Rect::new(lower[0], lower[1], upper[0] - lower[0], upper[1] - lower[1]),
                )
            })
    }

    pub fn overlaps(&self, rect: Rect) -> impl Iterator<Item = (Entity, Rect)> + '_ {
        self.rtree
            .locate_in_envelope_intersecting(&AABB::from_corners(
                [rect.x, rect.y],
                [rect.x + rect.w, rect.y + rect.h],
            ))
            .map(|item| {
                let lower = item.geom().lower();
                let upper = item.geom().upper();
                (
                    item.data,
                    Rect::new(lower[0], lower[1], upper[0] - lower[0], upper[1] - lower[1]),
                )
            })
    }

    pub fn collides(&self, entity: Entity) -> Option<impl Iterator<Item = Entity> + '_> {
        Some(self.collisions.get(&entity)?.iter().copied())
    }

    pub fn collides_with(&self, entity: Entity, other: Entity) -> bool {
        self.collisions
            .get(&entity)
            .map(|entities| entities.contains(&other))
            .unwrap_or_default()
    }
}
