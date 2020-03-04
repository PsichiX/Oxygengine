use crate::phase::Phase;
use core::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Transition<T>
where
    T: Clone,
{
    from: T,
    to: T,
    #[serde(default)]
    pub phase: Phase,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    time: Scalar,
}

impl<T> Transition<T>
where
    T: Clone,
{
    pub fn new(from: T, to: T, phase: Phase) -> Self {
        Self {
            from,
            to,
            phase,
            playing: false,
            time: 0.0,
        }
    }

    pub fn instant(value: T) -> Self {
        Self::new(
            value.clone(),
            value,
            Phase::point(1.0).expect("Could not create point phase for instant transition"),
        )
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn start(&mut self) {
        self.time = 0.0;
    }

    pub fn end(&mut self) {
        self.time = self.phase.duration();
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time.max(0.0).min(self.duration());
    }

    pub fn in_progress(&self) -> bool {
        self.time < self.phase.duration()
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn from(&self) -> &T {
        &self.from
    }

    pub fn to(&self) -> &T {
        &self.to
    }

    pub fn duration(&self) -> Scalar {
        self.phase.duration()
    }

    pub fn phase(&self) -> Scalar {
        self.sample(self.time)
    }

    pub fn sample(&self, time: Scalar) -> Scalar {
        self.phase.sample(time)
    }

    pub fn set(&mut self, value: T) {
        self.from = self.to.clone();
        self.to = value;
        self.time = 0.0;
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.playing {
            let duration = self.phase.duration();
            let time = self.time + delta_time;
            if time >= duration {
                self.playing = false;
            }
            self.time = time.max(0.0).min(duration);
        }
    }
}
