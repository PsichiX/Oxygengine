use crate::phase::Phase;
use core::Scalar;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Transition<T> {
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
            playing: false,
            time: phase.time_frame().start,
            phase,
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
        self.time = self.phase.time_frame().start;
    }

    pub fn end(&mut self) {
        self.time = self.phase.time_frame().end;
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time
            .max(self.phase.time_frame().start)
            .min(self.phase.time_frame().end);
    }

    pub fn in_progress(&self) -> bool {
        self.time < self.phase.time_frame().end
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn from(&self) -> &T {
        &self.from
    }

    pub fn from_mut(&mut self) -> &mut T {
        &mut self.from
    }

    pub fn to(&self) -> &T {
        &self.to
    }

    pub fn to_mut(&mut self) -> &mut T {
        &mut self.to
    }

    pub fn time_frame(&self) -> Range<Scalar> {
        self.phase.time_frame()
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
            let time_frame = self.phase.time_frame();
            let time = self.time + delta_time;
            if time >= time_frame.end {
                self.playing = false;
            }
            self.time = time.max(time_frame.start).min(time_frame.end);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SwitchTransition {
    value: bool,
    #[serde(default)]
    pub phase: Phase,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    time: Scalar,
}

impl SwitchTransition {
    pub fn new(value: bool, phase: Phase) -> Self {
        let time = if value {
            phase.time_frame().end
        } else {
            phase.time_frame().start
        };
        Self {
            value,
            phase,
            playing: false,
            time,
        }
    }

    pub fn instant(value: bool) -> Self {
        Self::new(
            value,
            Phase::point(1.0).expect("Could not create point phase for instant switch"),
        )
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn start(&mut self) {
        self.time = if self.value {
            self.phase.time_frame().start
        } else {
            self.phase.time_frame().end
        };
    }

    pub fn end(&mut self) {
        self.time = if self.value {
            self.phase.time_frame().end
        } else {
            self.phase.time_frame().start
        };
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time
            .max(self.phase.time_frame().start)
            .min(self.phase.time_frame().end);
    }

    pub fn in_progress(&self) -> bool {
        if self.value {
            self.time < self.phase.time_frame().end
        } else {
            self.time > self.phase.time_frame().start
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn value(&self) -> bool {
        self.value
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

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.playing {
            let time_frame = self.phase.time_frame();
            let time = if self.value {
                let time = self.time + delta_time;
                if time >= time_frame.end {
                    self.playing = false;
                }
                time
            } else {
                let time = self.time - delta_time;
                if time <= time_frame.start {
                    self.playing = false;
                }
                time
            };
            self.time = time.max(time_frame.start).min(time_frame.end);
        }
    }
}
