use crate::component::{CompositeSprite, CompositeSpriteAnimation, CompositeSurfaceCache};
use core::{
    app::AppLifeCycle,
    ecs::{Comp, Universe, WorldRef},
    Scalar,
};

pub type CompositeSpriteAnimationSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    Comp<&'a mut CompositeSprite>,
    Comp<&'a mut CompositeSpriteAnimation>,
    Comp<&'a mut CompositeSurfaceCache>,
);

pub fn composite_sprite_animation_system(universe: &mut Universe) {
    let (world, lifecycle, ..) =
        universe.query_resources::<CompositeSpriteAnimationSystemResources>();

    let dt = lifecycle.delta_time_seconds() as Scalar;
    for (_, (sprite, animation, cache)) in world
        .query::<(
            &mut CompositeSprite,
            &mut CompositeSpriteAnimation,
            Option<&mut CompositeSurfaceCache>,
        )>()
        .iter()
    {
        if animation.dirty {
            animation.dirty = false;
            if let Some((name, phase, _, _)) = &animation.current {
                if let Some(anim) = animation.animations.get(name) {
                    if let Some(frame) = anim.frames.get(*phase as usize) {
                        sprite.set_sheet_frame(Some((anim.sheet.clone(), frame.clone())));
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
