use crate::{
    component::{CompositeRenderable, CompositeSurfaceCache},
    composite_renderer::{Command, CompositeRenderer, Image},
};
use core::ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CompositeSurfaceCacheSystemCache {
    cached_surfaces: HashMap<Entity, String>,
}

pub type CompositeSurfaceCacheSystemResources<'a, CR> = (
    WorldRef,
    &'a mut CR,
    &'a EntityChanges,
    &'a mut CompositeSurfaceCacheSystemCache,
    Comp<&'a mut CompositeSurfaceCache>,
    Comp<&'a mut CompositeRenderable>,
);

pub fn composite_surface_cache_system<CR>(universe: &mut Universe)
where
    CR: CompositeRenderer + 'static,
{
    let (world, mut renderer, changes, mut cache, ..) =
        universe.query_resources::<CompositeSurfaceCacheSystemResources<CR>>();

    for entity in changes.despawned() {
        if let Some(name) = cache.cached_surfaces.remove(&entity) {
            renderer.destroy_surface(&name);
        }
    }

    for (entity, (surface, renderable)) in world
        .query::<(&mut CompositeSurfaceCache, &mut CompositeRenderable)>()
        .iter()
    {
        if surface.dirty {
            surface.dirty = false;
            if !renderer.has_surface(surface.name()) {
                renderer.create_surface(surface.name(), surface.width(), surface.height());
                cache
                    .cached_surfaces
                    .insert(entity, surface.name().to_owned());
            } else if let Some((width, height)) = renderer.get_surface_size(surface.name()) {
                if width != surface.width() || height != surface.height() {
                    renderer.destroy_surface(surface.name());
                    renderer.create_surface(surface.name(), surface.width(), surface.height());
                    cache
                        .cached_surfaces
                        .insert(entity, surface.name().to_owned());
                }
            }
            let commands = vec![
                Command::Store,
                Command::Draw(renderable.0.clone()),
                Command::Restore,
            ];
            if renderer.update_surface(surface.name(), commands).is_ok() {
                renderable.0 = Image {
                    image: surface.name().to_owned().into(),
                    source: None,
                    destination: None,
                    alignment: 0.0.into(),
                }
                .into();
            }
        }
    }
}
