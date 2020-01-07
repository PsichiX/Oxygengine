use core::ecs::{Component, FlaggedStorage, VecStorage};
use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum AudioSourceDirtyMode {
    None,
    Param,
    All,
}

#[derive(Debug, Clone)]
pub struct AudioSourceConfig {
    pub audio: Cow<'static, str>,
    pub streaming: bool,
    pub looped: bool,
    pub playback_rate: f32,
    pub volume: f32,
    pub play: bool,
}

impl AudioSourceConfig {
    pub fn new(audio: Cow<'static, str>) -> Self {
        Self {
            audio,
            streaming: false,
            looped: false,
            playback_rate: 1.0,
            volume: 1.0,
            play: false,
        }
    }

    pub fn audio(mut self, value: Cow<'static, str>) -> Self {
        self.audio = value;
        self
    }

    pub fn streaming(mut self, value: bool) -> Self {
        self.streaming = value;
        self
    }

    pub fn looped(mut self, value: bool) -> Self {
        self.looped = value;
        self
    }

    pub fn playback_rate(mut self, value: f32) -> Self {
        self.playback_rate = value;
        self
    }

    pub fn volume(mut self, value: f32) -> Self {
        self.volume = value;
        self
    }

    pub fn play(mut self, value: bool) -> Self {
        self.play = value;
        self
    }
}

#[derive(Debug, Clone)]
pub struct AudioSource {
    audio: Cow<'static, str>,
    streaming: bool,
    looped: bool,
    playback_rate: f32,
    volume: f32,
    play: bool,
    pub(crate) current_time: Option<f32>,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) dirty: AudioSourceDirtyMode,
}

impl From<AudioSourceConfig> for AudioSource {
    fn from(config: AudioSourceConfig) -> Self {
        Self::new_complex(
            config.audio,
            config.streaming,
            config.looped,
            config.playback_rate,
            config.volume,
            config.play,
        )
    }
}

impl AudioSource {
    pub fn new(audio: Cow<'static, str>, streaming: bool) -> Self {
        Self {
            audio,
            streaming,
            looped: false,
            playback_rate: 1.0,
            volume: 1.0,
            play: false,
            current_time: None,
            ready: Arc::new(AtomicBool::new(false)),
            dirty: AudioSourceDirtyMode::All,
        }
    }

    pub fn new_play(audio: Cow<'static, str>, streaming: bool, play: bool) -> Self {
        Self {
            audio,
            streaming,
            looped: false,
            playback_rate: 1.0,
            volume: 1.0,
            play,
            current_time: None,
            ready: Arc::new(AtomicBool::new(false)),
            dirty: AudioSourceDirtyMode::All,
        }
    }

    pub fn new_complex(
        audio: Cow<'static, str>,
        streaming: bool,
        looped: bool,
        playback_rate: f32,
        volume: f32,
        play: bool,
    ) -> Self {
        Self {
            audio,
            streaming,
            looped,
            playback_rate,
            volume,
            play,
            current_time: None,
            ready: Arc::new(AtomicBool::new(false)),
            dirty: AudioSourceDirtyMode::All,
        }
    }

    pub fn audio(&self) -> &str {
        &self.audio
    }

    pub fn streaming(&self) -> bool {
        self.streaming
    }

    pub fn looped(&self) -> bool {
        self.looped
    }

    pub fn set_looped(&mut self, looped: bool) {
        self.looped = looped;
        self.dirty = AudioSourceDirtyMode::Param;
    }

    pub fn playback_rate(&self) -> f32 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: f32) {
        self.playback_rate = playback_rate;
        self.dirty = AudioSourceDirtyMode::Param;
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.dirty = AudioSourceDirtyMode::Param;
    }

    pub fn current_time(&self) -> Option<f32> {
        self.current_time
    }

    pub fn must_play(&self) -> bool {
        self.play
    }

    pub fn play(&mut self) {
        self.play = true;
        self.dirty = AudioSourceDirtyMode::All;
    }

    pub fn stop(&mut self) {
        self.play = false;
        self.dirty = AudioSourceDirtyMode::All;
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

impl Component for AudioSource {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
