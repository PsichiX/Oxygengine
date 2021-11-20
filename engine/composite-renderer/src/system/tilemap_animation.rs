use crate::component::{CompositeSurfaceCache, CompositeTilemap, CompositeTilemapAnimation};
use core::{
    app::AppLifeCycle,
    ecs::{Comp, Universe, WorldRef},
    Scalar,
};

pub type CompositeTilemapAnimationSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    Comp<&'a mut CompositeTilemap>,
    Comp<&'a mut CompositeTilemapAnimation>,
    Comp<&'a mut CompositeSurfaceCache>,
);

pub fn composite_tilemap_animation_system(universe: &mut Universe) {
    let (world, lifecycle, ..) =
        universe.query_resources::<CompositeTilemapAnimationSystemResources>();

    let dt = lifecycle.delta_time_seconds() as Scalar;
    for (_, (tilemap, animation, cache)) in world
        .query::<(
            &mut CompositeTilemap,
            &mut CompositeTilemapAnimation,
            Option<&mut CompositeSurfaceCache>,
        )>()
        .iter()
    {
        if animation.dirty {
            animation.dirty = false;
            if let Some((name, phase, _, _)) = &animation.current {
                if let Some(anim) = animation.animations.get(name) {
                    if let Some(frame) = anim.frames.get(*phase as usize) {
                        tilemap.set_tileset(Some(anim.tileset.clone()));
                        tilemap.set_grid(frame.clone());
                        if let Some(cache) = cache {
                            cache.rebuild();
                        }
                    }
                }
            }
        }
        animation.process(dt);
    }
}
