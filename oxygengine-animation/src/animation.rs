use crate::{
    curve::{Curved, CurvedDistance, CurvedOffset},
    phase::Phase,
    spline::Spline,
};
use core::Scalar;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Animation<T>
where
    T: Default + Clone + Curved + CurvedOffset + CurvedDistance,
{
    pub value_spline: Spline<T>,
    #[serde(default)]
    pub time_phase: Phase,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    pub looped: bool,
    #[serde(default)]
    time: Scalar,
}

impl<T> Animation<T>
where
    T: Default + Clone + Curved + CurvedOffset + CurvedDistance,
{
    pub fn new(value_spline: Spline<T>, time_phase: Phase) -> Self {
        Self {
            value_spline,
            time_phase,
            playing: false,
            looped: false,
            time: 0.0,
        }
    }

    pub fn instant(value: T) -> Self {
        Self::new(
            Spline::point(value)
                .expect("Could not create point value spline for instant animation"),
            Phase::point(1.0).expect("Could not create point time phase for instant animation"),
        )
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn start(&mut self) {
        self.time = 0.0;
    }

    pub fn end(&mut self) {
        self.time = self.duration();
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time.max(0.0).min(self.duration());
    }

    pub fn in_progress(&self) -> bool {
        self.playing
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn duration(&self) -> Scalar {
        self.time_phase.duration()
    }

    pub fn value(&self) -> T {
        self.sample(self.time)
    }

    pub fn sample(&self, time: Scalar) -> T {
        self.value_spline.sample(self.time_phase.sample(time))
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.playing {
            let duration = self.duration();
            let mut time = self.time + delta_time;
            if time >= duration {
                if self.looped {
                    time = 0.0;
                } else {
                    self.playing = false;
                }
            }
            self.time = time.max(0.0).min(duration);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PhaseAnimation {
    pub value_phase: Phase,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    pub looped: bool,
    #[serde(default)]
    time: Scalar,
}

impl PhaseAnimation {
    pub fn new(value_phase: Phase) -> Self {
        Self {
            value_phase,
            playing: false,
            looped: false,
            time: 0.0,
        }
    }

    pub fn instant(value: Scalar) -> Self {
        Self::new(
            Phase::point(value).expect("Could not create point phase for instant phase animation"),
        )
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn start(&mut self) {
        self.time = 0.0;
    }

    pub fn end(&mut self) {
        self.time = self.duration();
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time.max(0.0).min(self.duration());
    }

    pub fn in_progress(&self) -> bool {
        self.playing
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn duration(&self) -> Scalar {
        self.value_phase.duration()
    }

    pub fn value(&self) -> Scalar {
        self.sample(self.time)
    }

    pub fn sample(&self, time: Scalar) -> Scalar {
        self.value_phase.sample(time)
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.playing {
            let duration = self.duration();
            let mut time = self.time + delta_time;
            if time >= duration {
                if self.looped {
                    time = 0.0;
                } else {
                    self.playing = false;
                }
            }
            self.time = time.max(0.0).min(duration);
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Interpolation<T>
where
    T: Default + Clone + Curved,
{
    #[serde(default)]
    pub from: T,
    #[serde(default)]
    pub to: T,
    #[serde(default)]
    pub time_phase: Phase,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    pub looped: bool,
    #[serde(default)]
    time: Scalar,
}

impl<T> Interpolation<T>
where
    T: Default + Clone + Curved,
{
    pub fn new(from: T, to: T, time_phase: Phase) -> Self {
        Self {
            from,
            to,
            time_phase,
            playing: false,
            looped: false,
            time: 0.0,
        }
    }

    pub fn instant(value: T) -> Self {
        Self::new(
            value.clone(),
            value,
            Phase::point(1.0).expect("Could not create point time phase for instant animation"),
        )
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn start(&mut self) {
        self.time = 0.0;
    }

    pub fn end(&mut self) {
        self.time = self.duration();
    }

    pub fn set_time(&mut self, time: Scalar) {
        self.time = time.max(0.0).min(self.duration());
    }

    pub fn in_progress(&self) -> bool {
        self.playing
    }

    pub fn is_complete(&self) -> bool {
        !self.in_progress()
    }

    pub fn duration(&self) -> Scalar {
        self.time_phase.duration()
    }

    pub fn value(&self) -> T {
        self.sample(self.time)
    }

    pub fn sample(&self, time: Scalar) -> T {
        self.from
            .interpolate(&self.to, self.time_phase.sample(time))
    }

    pub fn set(&mut self, value: T) {
        self.from = self.value();
        self.to = value;
        self.time = 0.0;
    }

    pub fn process(&mut self, delta_time: Scalar) {
        if self.playing {
            let duration = self.duration();
            let mut time = self.time + delta_time;
            if time >= duration {
                if self.looped {
                    time = 0.0;
                } else {
                    self.playing = false;
                }
            }
            self.time = time.max(0.0).min(duration);
        }
    }
}
