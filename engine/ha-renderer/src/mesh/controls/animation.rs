use crate::{
    asset_protocols::rig_animation::*,
    components::{
        rig_instance::{HaRigControlSignal, HaRigInstance},
        transform::HaTransform,
    },
};
use oxygengine_core::{
    assets::database::AssetsDatabase,
    scripting::intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    Scalar,
};
use std::collections::HashMap;

macro_rules! try_select_state {
    ($animation: expr, $control: expr, $state: expr) => {
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
                            $control
                                .property(k)
                                .managed()
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
                            $control
                                .property(k)
                                .managed()
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
                                            $control
                                                .property(n)
                                                .managed()
                                                .and_then(|v| v.read::<Scalar>().map(|v| *v))
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
    ($sequences: expr, $animation: expr, $control: expr, $speed: expr, $delta_time: expr, $change_scale: expr, $playing: expr) => {
        for (name, data) in $sequences {
            if let Some(sequence) = $animation.sequences.get(name) {
                if let Some(time_frame) = sequence.time_frame() {
                    let duration = time_frame.end - time_frame.start;
                    let delta = $speed * $animation.speed * sequence.speed * $delta_time;
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
                    $control.signals.extend(
                        sequence
                            .signals
                            .iter()
                            .filter(|signal| {
                                signal.time >= time_range.start && signal.time < time_range.end
                            })
                            .map(|signal| {
                                HaRigControlSignal::new(&signal.id).params(
                                    signal.params.iter().filter_map(|(key, value)| {
                                        Some((key.to_owned(), value.produce().ok()?))
                                    }),
                                )
                            }),
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
    ($state: expr, $active: expr, $control: expr) => {
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
                                $control
                                    .property(n)
                                    .managed()
                                    .and_then(|v| v.read::<Scalar>().map(|v| *v))
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

#[derive(IntuicioStruct)]
#[intuicio(name = "AnimationRigControl", module_name = "control_rig")]
pub struct AnimationRigControl {
    pub playing_property: String,
    pub speed_property: String,
    pub state_property: String,
    pub animation_asset_property: String,
    #[intuicio(ignore)]
    animation_asset: Option<String>,
    #[intuicio(ignore)]
    active: Option<Active>,
}

impl Default for AnimationRigControl {
    fn default() -> Self {
        Self {
            playing_property: "playing".to_owned(),
            speed_property: "speed".to_owned(),
            state_property: "state".to_owned(),
            animation_asset_property: "animation-asset".to_owned(),
            animation_asset: None,
            active: None,
        }
    }
}

impl AnimationRigControl {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::solve__define_function(registry));
    }
}

#[intuicio_methods(module_name = "control_rig")]
impl AnimationRigControl {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn solve(
        this: &mut Self,
        rig: &mut HaRigInstance,
        assets: &AssetsDatabase,
        delta_time: Scalar,
    ) {
        if let Some(animation_asset) = rig
            .control
            .property(&this.animation_asset_property)
            .consumed::<String>()
        {
            if this.animation_asset.is_none()
                || this.animation_asset.as_ref().unwrap() != &animation_asset
            {
                this.active = None;
                this.animation_asset = Some(format!("riganim://{}", animation_asset));
            }
        }
        let animation_asset = match &this.animation_asset {
            Some(animation_asset) => animation_asset,
            None => return,
        };
        let animation = match assets
            .asset_by_path(animation_asset)
            .and_then(|asset| asset.get::<RigAnimationAsset>())
        {
            Some(animation) => animation,
            None => return,
        };
        if let Some(state) = rig
            .control
            .property(&this.state_property)
            .consumed::<String>()
        {
            if animation.states.contains_key(&state) {
                this.active = Some(Active {
                    state,
                    current_sequences: Default::default(),
                    old_sequences: Default::default(),
                });
            } else {
                this.active = None;
            }
        }
        if this.active.is_none() {
            if let Some(state) = &animation.default_state {
                this.active = Some(Active {
                    state: state.to_owned(),
                    current_sequences: Default::default(),
                    old_sequences: Default::default(),
                });
            }
        }
        if !rig
            .control
            .property(&this.playing_property)
            .copied::<bool>()
            .unwrap_or(false)
        {
            return;
        }

        let speed = rig
            .control
            .property(&this.speed_property)
            .copied::<Scalar>()
            .unwrap_or(1.0);

        let active = match this.active.as_mut() {
            Some(active) => active,
            None => return,
        };

        if let Some(state) = animation.states.get(&active.state) {
            let selected = try_select_state!(animation, rig.control, state);
            try_apply_selected_state!(selected, active, animation);
        }

        if let Some(state) = animation.states.get(&active.state) {
            try_initialize_sequences!(active, state);
            let mut playing = false;
            update_sequences!(
                &mut active.current_sequences,
                animation,
                rig.control,
                speed,
                delta_time,
                1.0,
                playing
            );
            update_sequences!(
                &mut active.old_sequences,
                animation,
                rig.control,
                speed,
                delta_time,
                -1.0,
                playing
            );
            rig.control.property(&this.playing_property).set(playing);
            update_blend_weights!(state, active, rig.control);
        }

        active
            .old_sequences
            .retain(|_, data| data.change_time > 0.0);

        let total_weight = active
            .current_sequences
            .values()
            .chain(active.old_sequences.values())
            .fold(0.0, |a, v| a + v.weight());

        rig.skeleton
            .with_existing_bone_transforms(|name, transform| {
                let result = HaTransform::interpolate_many(
                    active
                        .current_sequences
                        .iter()
                        .chain(active.old_sequences.iter())
                        .map(|(sequence, data)| {
                            let transform = animation
                                .sequences
                                .get(sequence)
                                .map(|sequence| sequence.sample_bone(name, data.time, transform))
                                .unwrap_or_else(|| transform.to_owned());
                            let weight = data.weight() / total_weight;
                            (transform, weight)
                        }),
                );
                if let Some(result) = result {
                    *transform = result;
                }
            });
    }
}

#[derive(Debug, Clone)]
struct ActiveSequence {
    pub blend_weight: Scalar,
    pub time: Scalar,
    pub change_time: Scalar,
    pub change_time_limit: Scalar,
    pub bounced: bool,
}

impl ActiveSequence {
    pub fn change_weight(&self) -> Scalar {
        if self.change_time_limit > 0.0 {
            self.change_time / self.change_time_limit
        } else {
            1.0
        }
    }

    pub fn weight(&self) -> Scalar {
        self.blend_weight * self.change_weight()
    }
}

#[derive(Debug, Clone)]
struct Active {
    pub state: String,
    pub current_sequences: HashMap<String, ActiveSequence>,
    pub old_sequences: HashMap<String, ActiveSequence>,
}
