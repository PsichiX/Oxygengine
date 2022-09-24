use oxygengine_core::prelude::*;

pub(crate) struct AudioRequest {
    pub path: String,
    pub volume: Scalar,
}

#[derive(Default)]
pub struct AudioPlayer {
    pub(crate) queue: Vec<AudioRequest>,
    pub(crate) sources_count: usize,
}

impl AudioPlayer {
    pub fn play(&mut self, path: impl ToString, volume: Scalar) {
        self.queue.push(AudioRequest {
            path: path.to_string(),
            volume,
        });
    }

    pub fn sources_count(&self) -> usize {
        self.sources_count
    }
}
