use crate::{
    audio_asset_protocol::AudioAsset,
    component::{AudioSource, AudioSourceDirtyMode},
    resource::{Audio, AudioPlayState},
};
use core::{
    assets::database::AssetsDatabase,
    ecs::{life_cycle::EntityChanges, Comp, Universe, WorldRef},
};

pub type AudioSystemResources<'a, A> = (
    WorldRef,
    &'a EntityChanges,
    &'a AssetsDatabase,
    &'a mut A,
    Comp<&'a mut AudioSource>,
);

pub fn audio_system<A>(universe: &mut Universe)
where
    A: Audio + 'static,
{
    let (world, changes, assets, mut audio, ..) =
        universe.query_resources::<AudioSystemResources<A>>();

    audio.update_cache(&assets);

    for entity in changes.despawned() {
        audio.destroy_source(entity);
    }

    for (entity, source) in world.query::<&mut AudioSource>().iter() {
        if source.dirty != AudioSourceDirtyMode::None {
            if !audio.has_source(entity) {
                if let Some(id) = audio.get_asset_id(source.audio()) {
                    if let Some(asset) = assets.asset_by_id(id) {
                        if let Some(asset) = asset.get::<AudioAsset>() {
                            audio.create_source(
                                entity,
                                asset.bytes(),
                                source.streaming(),
                                source.looped(),
                                source.playback_rate(),
                                source.volume(),
                                source.is_playing(),
                                source.ready.clone(),
                            );
                            source.dirty = AudioSourceDirtyMode::None;
                        }
                    }
                }
            } else {
                audio.update_source(
                    entity,
                    source.looped(),
                    source.playback_rate(),
                    source.volume(),
                    if source.dirty == AudioSourceDirtyMode::All {
                        Some(source.is_playing())
                    } else {
                        None
                    },
                );
                source.dirty = AudioSourceDirtyMode::None;
            }
        }
        if let Some(state) = audio.get_source_state(entity) {
            source.current_time = state.current_time;
            match state.is_playing {
                AudioPlayState::Ended(v) => {
                    if v {
                        source.stop();
                    }
                }
                AudioPlayState::State(v) => {
                    if v != source.is_playing() {
                        if v {
                            source.play();
                        } else {
                            source.stop();
                        }
                    }
                }
            }
        }
    }
}
