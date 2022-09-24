use crate::resources::audio_player::*;
use oxygengine_audio::prelude::*;
use oxygengine_core::prelude::*;

pub type AudioPlayerResources<'a> = (
    WorldRef,
    &'a mut UniverseCommands,
    &'a mut AudioPlayer,
    Comp<&'a AudioSource>,
);

struct AudioPlayerTag;

pub fn audio_player_system(universe: &mut Universe) {
    let (world, mut commands, mut audio, ..) = universe.query_resources::<AudioPlayerResources>();

    for request in audio.queue.drain(..) {
        commands.schedule(SpawnEntity::from_bundle((
            AudioSource::new_complex(request.path.into(), false, false, 1.0, request.volume, true),
            AudioPlayerTag,
        )));
    }

    audio.sources_count = 0;
    for (entity, source) in world
        .query::<&AudioSource>()
        .with::<&AudioPlayerTag>()
        .iter()
    {
        if !source.is_playing() {
            commands.schedule(DespawnEntity(entity));
        } else {
            audio.sources_count += 1;
        }
    }
}
