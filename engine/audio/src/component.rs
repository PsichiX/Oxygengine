use core::{
    ecs::Entity,
    prefab::{Prefab, PrefabError, PrefabProxy},
    state::StateToken,
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AudioSourceDirtyMode {
    None,
    Param,
    All,
}

impl Default for AudioSourceDirtyMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSourceConfig {
    pub audio: Cow<'static, str>,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub looped: bool,
    #[serde(default = "AudioSourceConfig::default_playback_rate")]
    pub playback_rate: Scalar,
    #[serde(default = "AudioSourceConfig::default_volume")]
    pub volume: Scalar,
    #[serde(default)]
    pub play: bool,
}

impl AudioSourceConfig {
    fn default_playback_rate() -> Scalar {
        1.0
    }

    fn default_volume() -> Scalar {
        1.0
    }

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

    pub fn playback_rate(mut self, value: Scalar) -> Self {
        self.playback_rate = value;
        self
    }

    pub fn volume(mut self, value: Scalar) -> Self {
        self.volume = value;
        self
    }

    pub fn play(mut self, value: bool) -> Self {
        self.play = value;
        self
    }
}

impl Prefab for AudioSourceConfig {}

#[derive(Debug, Clone)]
pub struct AudioSource {
    audio: Cow<'static, str>,
    streaming: bool,
    looped: bool,
    playback_rate: Scalar,
    volume: Scalar,
    play: bool,
    pub(crate) current_time: Option<Scalar>,

    pub(crate) ready: Arc<AtomicBool>,

    pub(crate) dirty: AudioSourceDirtyMode,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            audio: "".into(),
            streaming: false,
            looped: false,
            playback_rate: 1.0,
            volume: 1.0,
            play: false,
            current_time: None,
            ready: Arc::new(AtomicBool::new(false)),
            dirty: AudioSourceDirtyMode::None,
        }
    }
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
        playback_rate: Scalar,
        volume: Scalar,
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
        self.dirty = self.dirty.max(AudioSourceDirtyMode::Param);
    }

    pub fn playback_rate(&self) -> Scalar {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: Scalar) {
        self.playback_rate = playback_rate;
        self.dirty = self.dirty.max(AudioSourceDirtyMode::Param);
    }

    pub fn volume(&self) -> Scalar {
        self.volume
    }

    pub fn set_volume(&mut self, volume: Scalar) {
        self.volume = volume;
        self.dirty = self.dirty.max(AudioSourceDirtyMode::Param);
    }

    pub fn current_time(&self) -> Option<Scalar> {
        self.current_time
    }

    pub fn is_playing(&self) -> bool {
        self.play
    }

    pub fn play(&mut self) {
        self.play = true;
        self.dirty = self.dirty.max(AudioSourceDirtyMode::All);
    }

    pub fn stop(&mut self) {
        self.play = false;
        self.dirty = self.dirty.max(AudioSourceDirtyMode::All);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

pub type AudioSourcePrefabProxy = AudioSourceConfig;

impl PrefabProxy<AudioSourcePrefabProxy> for AudioSource {
    fn from_proxy_with_extras(
        proxy: AudioSourcePrefabProxy,
        _: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        Ok(proxy.into())
    }
}
