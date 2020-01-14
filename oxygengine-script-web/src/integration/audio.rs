use crate::interface::ComponentModify;
use oxygengine_audio::component::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AudioSourceScripted {
    pub audio: String,
    pub streaming: bool,
    pub looped: bool,
    pub playback_rate: f32,
    pub volume: f32,
    pub play: bool,
    pub current_time: Option<f32>,
    pub ready: bool,
}

impl From<AudioSource> for AudioSourceScripted {
    fn from(value: AudioSource) -> Self {
        Self {
            audio: value.audio().into(),
            streaming: value.streaming(),
            looped: value.looped(),
            playback_rate: value.playback_rate(),
            volume: value.volume(),
            play: value.must_play(),
            current_time: value.current_time(),
            ready: value.is_ready(),
        }
    }
}

impl ComponentModify<AudioSourceScripted> for AudioSource {
    fn modify_component(&mut self, source: AudioSourceScripted) {
        if self.audio() != source.audio {
            *self = Self::new_complex(
                source.audio.into(),
                source.streaming,
                source.looped,
                source.playback_rate,
                source.volume,
                source.play,
            );
            return;
        }
        self.set_looped(source.looped);
        self.set_playback_rate(source.playback_rate);
        self.set_volume(source.volume);
        if !self.must_play() && source.play {
            self.play();
        } else if self.must_play() && !source.play {
            self.stop();
        }
    }
}
