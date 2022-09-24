extern crate oxygengine_audio as audio;
extern crate oxygengine_backend_web as web;
extern crate oxygengine_core as core;

use audio::resource::*;
use core::{
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::Entity,
    Scalar,
};
use futures::{future, TryFutureExt};
use js_sys::*;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web::closure::WebClosure;
use web_sys::*;

pub mod prelude {
    pub use crate::*;
}

#[derive(Debug)]
enum AudioCache {
    /// (source, (gain, notify ended))
    Buffered(
        AudioBufferSourceNode,
        (GainNode, Arc<AtomicBool>, WebClosure),
    ),
    Streaming(HtmlAudioElement, MediaElementAudioSourceNode),
}

pub struct WebAudio {
    context: AudioContext,
    table_forward: HashMap<String, AssetId>,
    table_backward: HashMap<AssetId, String>,
    sources_cache: HashMap<Entity, AudioCache>,
}

unsafe impl Send for WebAudio {}
unsafe impl Sync for WebAudio {}

impl Default for WebAudio {
    fn default() -> Self {
        Self {
            context: AudioContext::new().unwrap(),
            table_forward: Default::default(),
            table_backward: Default::default(),
            sources_cache: Default::default(),
        }
    }
}

impl Audio for WebAudio {
    fn create_source(
        &mut self,
        entity: Entity,
        data: &[u8],
        streaming: bool,
        looped: bool,
        playback_rate: Scalar,
        volume: Scalar,
        play: bool,
        notify_ready: Arc<AtomicBool>,
    ) {
        let buffer = Uint8Array::from(data);
        let cache = if streaming {
            let buffer_val: &JsValue = buffer.as_ref();
            let parts = Array::new_with_length(1);
            parts.set(0, buffer_val.clone());
            let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
            let audio = HtmlAudioElement::new().unwrap();
            audio.set_src(&Url::create_object_url_with_blob(&blob).unwrap());
            let node = self
                .context
                .create_media_element_source(audio.as_ref())
                .unwrap();
            node.connect_with_audio_node(&self.context.destination())
                .expect("Could not connect audio stream source with audio output");
            audio.load();
            audio.set_loop(looped);
            audio.set_playback_rate(playback_rate as f64);
            audio.set_volume(volume as f64);
            if play {
                if self.context.state() != AudioContextState::Running {
                    drop(self.context.resume());
                }
                audio.set_current_time(0.0);
                drop(audio.play().expect("Could not start audio source"));
            }
            notify_ready.store(true, Ordering::Relaxed);
            AudioCache::Streaming(audio, node)
        } else {
            let audio = self.context.create_buffer_source().unwrap();
            let notify_ended = Arc::new(AtomicBool::new(false));
            let notify_ended2 = notify_ended.clone();
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                notify_ended.store(true, Ordering::Relaxed);
            }) as Box<dyn FnMut(_)>);
            // audio.set_onended(Some(closure.as_ref().unchecked_ref()));
            audio
                .add_event_listener_with_callback("ended", closure.as_ref().unchecked_ref())
                .unwrap();
            let ended_closure = WebClosure::acquire(closure);
            let audio2 = audio.clone();
            let gain = self.context.create_gain().unwrap();
            let gain2 = gain.clone();
            let destination = self.context.destination();
            let context = self.context.clone();
            let promise = self.context.decode_audio_data(&buffer.buffer()).unwrap();
            let future = JsFuture::from(promise).and_then(move |buff| {
                assert!(buff.is_instance_of::<AudioBuffer>());
                let buff: AudioBuffer = buff.dyn_into().unwrap();
                audio
                    .connect_with_audio_node(gain.as_ref())
                    .expect("Could not connect audio source with gain");
                gain.connect_with_audio_node(destination.as_ref())
                    .expect("Could not connect gain with audio output");
                audio.set_buffer(Some(&buff));
                audio.set_loop(looped);
                audio.playback_rate().set_value(playback_rate as f32);
                gain.gain().set_value(volume as f32);
                if play {
                    if context.state() != AudioContextState::Running {
                        drop(context.resume());
                    }
                    audio.start().expect("Could not start audio source");
                }
                notify_ready.store(true, Ordering::Relaxed);
                future::ok(JsValue::null())
            });
            // TODO: fail process on error catch.
            drop(future_to_promise(future));
            AudioCache::Buffered(audio2, (gain2, notify_ended2, ended_closure))
        };
        self.sources_cache.insert(entity, cache);
    }

    fn destroy_source(&mut self, entity: Entity) {
        if let Some(audio) = self.sources_cache.remove(&entity) {
            match audio {
                AudioCache::Buffered(audio, (gain, notify_ended, mut ended_closure)) => {
                    audio
                        .disconnect()
                        .expect("Could not disconnect audio source from gain");
                    gain.disconnect()
                        .expect("Could not disconnect gain from audio output");
                    notify_ended.store(true, Ordering::Relaxed);
                    ended_closure.release();
                }
                AudioCache::Streaming(_, audio) => audio
                    .disconnect()
                    .expect("Could not disconnect audio stream source from audio output"),
            }
        }
    }

    fn has_source(&mut self, entity: Entity) -> bool {
        self.sources_cache.contains_key(&entity)
    }

    fn update_source(
        &mut self,
        entity: Entity,
        looped: bool,
        playback_rate: Scalar,
        volume: Scalar,
        play: Option<bool>,
    ) {
        if let Some(audio) = self.sources_cache.get(&entity) {
            match audio {
                AudioCache::Buffered(audio, (gain, _, _)) => {
                    if audio.buffer().is_some() {
                        audio.set_loop(looped);
                        audio.playback_rate().set_value(playback_rate as f32);
                        gain.gain().set_value(volume as f32);
                        if let Some(play) = play {
                            if play {
                                audio.start().expect("Could not start audio source");
                            } else {
                                audio.stop().expect("Could not stop audio source");
                            }
                        }
                    }
                }
                AudioCache::Streaming(audio, _) => {
                    audio.set_loop(looped);
                    audio.set_playback_rate(playback_rate as f64);
                    audio.set_volume(volume as f64);
                    if let Some(play) = play {
                        if play {
                            if self.context.state() != AudioContextState::Running {
                                drop(self.context.resume());
                            }
                            audio.set_current_time(0.0);
                            drop(audio.play().expect("Could not start audio source"));
                        } else {
                            audio.pause().expect("Could not stop audio source");
                        }
                    }
                }
            }
        }
    }

    fn get_source_state(&self, entity: Entity) -> Option<AudioState> {
        self.sources_cache.get(&entity).map(|audio| match audio {
            AudioCache::Buffered(_, (_, notify_ended, _)) => AudioState {
                current_time: None,
                is_playing: AudioPlayState::Ended(notify_ended.swap(false, Ordering::Relaxed)),
            },
            AudioCache::Streaming(audio, _) => AudioState {
                current_time: Some(audio.current_time() as Scalar),
                is_playing: AudioPlayState::State(!audio.paused()),
            },
        })
    }

    fn get_asset_id(&self, path: &str) -> Option<AssetId> {
        self.table_forward.get(path).copied()
    }

    fn update_cache(&mut self, assets: &AssetsDatabase) {
        for id in assets.lately_loaded_protocol("audio") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded audio asset");
            let path = asset.path().to_owned();
            self.table_forward.insert(path.clone(), id);
            self.table_backward.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("audio") {
            if let Some(path) = self.table_backward.remove(id) {
                self.table_forward.remove(&path);
            }
        }
    }
}
