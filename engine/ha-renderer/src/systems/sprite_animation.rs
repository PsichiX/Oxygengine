use crate::{asset_protocols::sprite_animation::*, components::sprite_animation_instance::*};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
    Scalar,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaSpriteAnimationSystemCache {
    map: HashMap<String, (SpriteAnimationAsset, AssetId)>,
    table: HashMap<AssetId, String>,
}

pub type HaSpriteAnimationSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a mut HaSpriteAnimationSystemCache,
    Comp<&'a mut HaSpriteAnimationInstance>,
);

macro_rules! select_state {
    ($animation: expr, $sprite: expr, $state: expr, $next: expr) => {
        $animation
            .rules
            .iter()
            .chain($state.rules.iter())
            .find_map(|rule| {
                match rule {
                    SpriteAnimationRule::Single {
                        target_state,
                        conditions,
                        region,
                    } => {
                        if region.contains($next)
                            && conditions.iter().all(|(k, c)| {
                                $sprite
                                    .values
                                    .get(k)
                                    .map(|v| c.validate(v))
                                    .unwrap_or_default()
                            })
                        {
                            return Some(target_state);
                        }
                    }
                    SpriteAnimationRule::BlendSpace {
                        conditions,
                        axis_scalars,
                        blend_states,
                    } => {
                        if conditions.iter().all(|(k, c)| {
                            $sprite
                                .values
                                .get(k)
                                .map(|v| c.validate(v))
                                .unwrap_or_default()
                        }) {
                            return blend_states
                                .iter()
                                .map(|state| {
                                    let result = state
                                        .axis_values
                                        .iter()
                                        .zip(axis_scalars.iter().map(|n| {
                                            $sprite
                                                .values
                                                .get(n)
                                                .and_then(|v| v.as_scalar())
                                                .unwrap_or_default()
                                        }))
                                        .map(|(a, b)| {
                                            let v = a - b;
                                            v * v
                                        })
                                        .fold(0.0, |a, v| a + v);
                                    (&state.target_state, result)
                                })
                                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                                .map(|(n, _)| n);
                        }
                    }
                }
                None
            })
    };
}

macro_rules! process_change {
    ($animation: expr, $active: expr, $sprite: expr, $dt: expr) => {
        if let Some(state) = $animation.states.get(&$active.state) {
            let length = state.frames.len() as Scalar;
            let delta = $sprite.speed * $animation.speed * state.speed * $dt;
            let prev = $active.frame as usize;
            let time_before = $active.frame;
            let end = if $active.bounced {
                $active.frame -= delta;
                $active.frame <= 0.0
            } else {
                $active.frame += delta;
                $active.frame >= length
            };
            let time_after = $active.frame;
            let time_range = time_before.min(time_after)..time_before.max(time_after);
            $sprite.signals.extend(
                state
                    .signals
                    .iter()
                    .filter(|signal| {
                        signal.time >= time_range.start && signal.time < time_range.end
                    })
                    .cloned(),
            );
            if end {
                if state.looping {
                    if state.bounce {
                        $active.bounced = !$active.bounced;
                    } else if $active.bounced {
                        $active.frame += length;
                    } else {
                        $active.frame -= length;
                    }
                } else {
                    $sprite.playing = false;
                }
            }
            $active.frame = $active.frame.max(0.0).min(length);
            let next = ($active.frame as usize).min(state.frames.len() - 1);
            let selected = select_state!($animation, $sprite, state, next);

            if let Some(selected) = selected {
                if &$active.state != selected {
                    $active.state = selected.to_owned();
                    $active.frame = 0.0;
                    $active.bounced = false;
                    $active.cached_frame = None;
                    true
                } else {
                    prev != next
                }
            } else {
                prev != next
            }
        } else {
            false
        }
    };
}

pub fn ha_sprite_animation(universe: &mut Universe) {
    let (world, lifecycle, assets, mut cache, ..) =
        universe.query_resources::<HaSpriteAnimationSystemResources>();

    for id in assets.lately_loaded_protocol("spriteanim") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<SpriteAnimationAsset>() {
                cache.map.insert(path.to_owned(), (asset.to_owned(), *id));
                cache.table.insert(*id, path.to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("spriteanim") {
        if let Some(name) = cache.table.remove(id) {
            cache.map.remove(&name);
        }
    }

    let dt = lifecycle.delta_time_seconds();
    for (_, sprite) in world.query::<&mut HaSpriteAnimationInstance>().iter() {
        sprite.frame_changed = false;
        sprite.signals.clear();

        if let Some((animation, _)) = cache.map.get(&sprite.animation) {
            if sprite.playing && sprite.active.is_none() {
                if let Some(name) = &animation.default_state {
                    sprite.play(name);
                }
            }

            if let (true, Some(active)) = (sprite.playing, sprite.active.as_mut()) {
                let change = process_change!(animation, active, sprite, dt);

                if let (true, Some(state)) = (change, animation.states.get(&active.state)) {
                    let index = (active.frame.max(0.0) as usize).min(state.frames.len() - 1);
                    if let Some(name) = state.frames.get(index) {
                        active.cached_frame = Some(name.to_owned());
                        sprite.frame_changed = true;
                    }
                }
            }
        }
    }
}
