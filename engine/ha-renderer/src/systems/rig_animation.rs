use crate::{asset_protocols::rig_animation::*, components::rig_animation_instance::*};
use core::{
    app::AppLifeCycle,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{Comp, Universe, WorldRef},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HaRigAnimationSystemCache {
    pub(crate) map: HashMap<String, (RigAnimationAsset, AssetId)>,
    table: HashMap<AssetId, String>,
}

pub type HaRigAnimationSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a AssetsDatabase,
    &'a mut HaRigAnimationSystemCache,
    Comp<&'a mut HaRigAnimationInstance>,
);

macro_rules! try_select_state {
    ($animation: expr, $skeleton: expr, $state: expr) => {
        $animation
            .rules
            .iter()
            .chain($state.rules.iter())
            .find_map(|rule| {
                match rule {
                    RigAnimationRule::Single {
                        target_state,
                        conditions,
                        change_time,
                    } => {
                        if conditions.iter().all(|(k, c)| {
                            $skeleton
                                .values
                                .get(k)
                                .map(|v| c.validate(v))
                                .unwrap_or_default()
                        }) {
                            return Some((target_state, *change_time));
                        }
                    }
                    RigAnimationRule::BlendSpace {
                        conditions,
                        axis_scalars,
                        blend_states,
                        change_time,
                    } => {
                        if conditions.iter().all(|(k, c)| {
                            $skeleton
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
                                            $skeleton
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
                                .map(|(n, _)| (n, *change_time));
                        }
                    }
                }
                None
            })
    };
}

macro_rules! try_apply_selected_state {
    ($selected: expr, $active: expr, $animation: expr) => {
        if let Some((name, time_limit)) = $selected {
            if &$active.state != name {
                $active.state = name.to_owned();
                for (name, data) in $active.current_sequences.drain() {
                    let change_time = time_limit * (data.change_time / data.change_time_limit);
                    if let Some(data) = $active.old_sequences.get_mut(&name) {
                        data.change_time = change_time;
                        data.change_time_limit = time_limit;
                    } else {
                        $active.old_sequences.insert(
                            name,
                            ActiveSequence {
                                blend_weight: data.blend_weight,
                                time: data.time,
                                change_time,
                                change_time_limit: time_limit,
                                bounced: data.bounced,
                            },
                        );
                    }
                }
                if let Some(state) = $animation.states.get(name) {
                    match &state.sequences {
                        RigAnimationStateSequences::Single(name) => {
                            if let Some(data) = $active.current_sequences.get_mut(name) {
                                let change_time =
                                    time_limit * (data.change_time / data.change_time_limit);
                                data.change_time = change_time;
                                data.change_time_limit = time_limit;
                            } else {
                                $active.current_sequences.insert(
                                    name.to_owned(),
                                    ActiveSequence {
                                        blend_weight: 1.0,
                                        time: 0.0,
                                        change_time: time_limit,
                                        change_time_limit: time_limit,
                                        bounced: false,
                                    },
                                );
                            }
                        }
                        RigAnimationStateSequences::BlendSpace { sequences, .. } => {
                            for blend_space in sequences {
                                if let Some(data) =
                                    $active.current_sequences.get_mut(&blend_space.sequence)
                                {
                                    let change_time =
                                        time_limit * (data.change_time / data.change_time_limit);
                                    data.change_time = change_time;
                                    data.change_time_limit = time_limit;
                                } else {
                                    $active.current_sequences.insert(
                                        blend_space.sequence.to_owned(),
                                        ActiveSequence {
                                            blend_weight: 1.0,
                                            time: 0.0,
                                            change_time: time_limit,
                                            change_time_limit: time_limit,
                                            bounced: false,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    };
}

macro_rules! try_initialize_sequences {
    ($active: expr, $state: expr) => {
        if $active.current_sequences.is_empty() {
            match &$state.sequences {
                RigAnimationStateSequences::Single(name) => {
                    if let Some(data) = $active.current_sequences.get_mut(name) {
                        data.change_time = 0.0;
                        data.change_time_limit = 0.0;
                    } else {
                        $active.current_sequences.insert(
                            name.to_owned(),
                            ActiveSequence {
                                blend_weight: 1.0,
                                time: 0.0,
                                change_time: 0.0,
                                change_time_limit: 0.0,
                                bounced: false,
                            },
                        );
                    }
                }
                RigAnimationStateSequences::BlendSpace { sequences, .. } => {
                    for blend_space in sequences {
                        if let Some(data) = $active.current_sequences.get_mut(&blend_space.sequence)
                        {
                            data.change_time = 0.0;
                            data.change_time_limit = 0.0;
                        } else {
                            $active.current_sequences.insert(
                                blend_space.sequence.to_owned(),
                                ActiveSequence {
                                    blend_weight: 1.0,
                                    time: 0.0,
                                    change_time: 0.0,
                                    change_time_limit: 0.0,
                                    bounced: false,
                                },
                            );
                        }
                    }
                }
            }
        }
    };
}

macro_rules! update_sequences {
    ($sequences: expr, $animation: expr, $skeleton: expr, $dt: expr, $change_scale: expr, $playing: expr) => {
        for (name, data) in $sequences {
            if let Some(sequence) = $animation.sequences.get(name) {
                if let Some(time_frame) = sequence.time_frame() {
                    let duration = time_frame.end - time_frame.start;
                    let delta = $skeleton.speed * $animation.speed * sequence.speed * $dt;
                    let time_before = data.time;
                    data.change_time = (data.change_time + delta * $change_scale)
                        .max(0.0)
                        .min(data.change_time_limit);
                    let end = if data.bounced {
                        data.time -= delta;
                        data.time <= time_frame.start
                    } else {
                        data.time += delta;
                        data.time >= time_frame.end
                    };
                    let time_after = data.time;
                    let time_range = time_before.min(time_after)..time_before.max(time_after);
                    $skeleton.signals.extend(
                        sequence
                            .signals
                            .iter()
                            .filter(|signal| {
                                signal.time >= time_range.start && signal.time < time_range.end
                            })
                            .cloned(),
                    );
                    if end {
                        if sequence.looping {
                            if sequence.bounce {
                                data.bounced = !data.bounced;
                            } else if data.bounced {
                                data.time += duration;
                            } else {
                                data.time -= duration;
                            }
                            $playing = true;
                        }
                    } else {
                        $playing = true;
                    }
                    data.time = data.time.max(time_frame.start).min(time_frame.end);
                }
            }
        }
    };
}

macro_rules! update_blend_weights {
    ($state: expr, $active: expr, $skeleton: expr) => {
        match &$state.sequences {
            RigAnimationStateSequences::Single(sequence) => {
                if let Some(data) = $active.current_sequences.get_mut(sequence) {
                    data.blend_weight = 1.0;
                }
            }
            RigAnimationStateSequences::BlendSpace {
                axis_scalars,
                sequences,
            } => {
                let mut total_blend_weight = 0.0;
                for blend_space in sequences {
                    if let Some(data) = $active.current_sequences.get_mut(&blend_space.sequence) {
                        data.blend_weight = blend_space
                            .axis_values
                            .iter()
                            .zip(axis_scalars.iter().map(|n| {
                                $skeleton
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
                        total_blend_weight += data.blend_weight;
                    }
                }
                for blend_space in sequences {
                    if let Some(data) = $active.current_sequences.get_mut(&blend_space.sequence) {
                        data.blend_weight = if total_blend_weight > 0.0 {
                            1.0 - (data.blend_weight / total_blend_weight)
                        } else {
                            1.0
                        };
                    }
                }
            }
        }
    };
}

pub fn ha_rig_animation(universe: &mut Universe) {
    let (world, lifecycle, assets, mut cache, ..) =
        universe.query_resources::<HaRigAnimationSystemResources>();

    for id in assets.lately_loaded_protocol("riganim") {
        if let Some(asset) = assets.asset_by_id(*id) {
            let path = asset.path();
            if let Some(asset) = asset.get::<RigAnimationAsset>() {
                cache.map.insert(path.to_owned(), (asset.to_owned(), *id));
                cache.table.insert(*id, path.to_owned());
            }
        }
    }
    for id in assets.lately_unloaded_protocol("riganim") {
        if let Some(name) = cache.table.remove(id) {
            cache.map.remove(&name);
        }
    }

    let dt = lifecycle.delta_time_seconds();
    for (_, rig) in world.query::<&mut HaRigAnimationInstance>().iter() {
        rig.signals.clear();

        if let Some((animation, _)) = cache.map.get(&rig.animation) {
            if rig.playing && rig.active.is_none() {
                if let Some(name) = &animation.default_state {
                    rig.play(name);
                }
            }

            if let (true, Some(active)) = (rig.playing, rig.active.as_mut()) {
                if let Some(state) = animation.states.get(&active.state) {
                    let selected = try_select_state!(animation, rig, state);
                    try_apply_selected_state!(selected, active, animation);
                }

                if let Some(state) = animation.states.get(&active.state) {
                    try_initialize_sequences!(active, state);
                    let mut playing = false;
                    update_sequences!(
                        &mut active.current_sequences,
                        animation,
                        rig,
                        dt,
                        1.0,
                        playing
                    );
                    update_sequences!(&mut active.old_sequences, animation, rig, dt, -1.0, playing);
                    rig.playing = playing;
                    update_blend_weights!(state, active, rig);
                }

                active
                    .old_sequences
                    .retain(|_, data| data.change_time > 0.0);
            }
        }
    }
}
