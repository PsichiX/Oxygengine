#![allow(clippy::type_complexity)]

use crate::{
    audio_asset_protocol::AudioAsset,
    component::{AudioSource, AudioSourceDirtyMode},
    resource::Audio,
};
use core::{
    assets::database::AssetsDatabase,
    ecs::{
        storage::ComponentEvent, Entities, Entity, Join, Read, ReaderId, Resources, System, Write,
        WriteStorage,
    },
};
use std::{collections::HashSet, marker::PhantomData};

pub struct AudioSystem<A>
where
    A: Audio,
{
    cached_sources: HashSet<Entity>,
    reader_id: Option<ReaderId<ComponentEvent>>,
    _phantom: PhantomData<A>,
}

impl<A> Default for AudioSystem<A>
where
    A: Audio,
{
    fn default() -> Self {
        Self {
            cached_sources: HashSet::new(),
            reader_id: None,
            _phantom: PhantomData,
        }
    }
}

impl<'s, A> System<'s> for AudioSystem<A>
where
    A: Audio + 'static,
{
    type SystemData = (
        Entities<'s>,
        Option<Write<'s, A>>,
        Option<Read<'s, AssetsDatabase>>,
        WriteStorage<'s, AudioSource>,
    );

    fn setup(&mut self, res: &mut Resources) {
        use core::ecs::SystemData;
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<AudioSource>::fetch(&res).register_reader());
    }

    fn run(&mut self, (entities, audio, assets, mut sources): Self::SystemData) {
        if audio.is_none() || assets.is_none() {
            return;
        }

        let audio: &mut A = &mut audio.unwrap();
        let assets: &AssetsDatabase = &assets.unwrap();
        audio.update_cache(assets);

        let events = sources.channel().read(self.reader_id.as_mut().unwrap());
        for event in events {
            if let ComponentEvent::Removed(index) = event {
                let found = self.cached_sources.iter().find_map(|entity| {
                    if entity.id() == *index {
                        Some(*entity)
                    } else {
                        None
                    }
                });
                if let Some(entity) = found {
                    self.cached_sources.remove(&entity);
                    audio.destroy_source(entity);
                }
            }
        }

        for (entity, source) in (&entities, &mut sources).join() {
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
                                    source.must_play(),
                                    source.ready.clone(),
                                );
                                self.cached_sources.insert(entity);
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
                            Some(source.must_play())
                        } else {
                            None
                        },
                    );
                    source.dirty = AudioSourceDirtyMode::None;
                    if let Some(state) = audio.get_source_state(entity) {
                        source.current_time = state.current_time;
                    }
                }
            }
        }
    }
}
