use crate::{
    utils::{diffuse, diffuse_with_barriers, remap_in_ranges, transfer},
    world_2d::{World2dField, World2dSimulation},
};
use oxygengine_utils::grid_2d::Grid2d;
use std::{any::Any, f64::consts::PI, ops::Range};

#[derive(Debug, Clone)]
pub struct World2dClimateSimulationConfig {
    pub humidity_limit_range: Range<f64>,
    pub water_capacity: f64,
    pub rainfall_rate: f64,
    pub evaporation_rate: f64,
    pub temperature_transfer_rate: f64,
    pub temperature_range: Range<f64>,
    pub world_axis_angle: f64,
    pub full_year_steps: usize,
}

impl Default for World2dClimateSimulationConfig {
    fn default() -> Self {
        Self {
            humidity_limit_range: 0.05..0.2,
            water_capacity: 100.0,
            rainfall_rate: 0.1,
            evaporation_rate: 0.01,
            temperature_transfer_rate: 0.01,
            temperature_range: -50.0..50.0,
            world_axis_angle: 20.0 * PI / 180.0,
            full_year_steps: 364 * 24,
        }
    }
}

#[derive(Default)]
pub struct World2dClimateSimulation {
    config: World2dClimateSimulationConfig,
    steps: usize,
    years: usize,
}

impl World2dClimateSimulation {
    pub fn new(config: World2dClimateSimulationConfig) -> Self {
        Self {
            config,
            steps: 0,
            years: 0,
        }
    }

    pub fn config(&self) -> &World2dClimateSimulationConfig {
        &self.config
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn years(&self) -> usize {
        self.years
    }

    fn surface_water_transfer(&self, altitude: &Grid2d<f64>, surface_water: &mut World2dField) {
        let surface_water = surface_water.iterate().unwrap();
        diffuse_with_barriers(altitude, surface_water.0, surface_water.1);
    }

    fn humidity_transfer(&self, humidity: &mut World2dField) {
        let humidity = humidity.iterate().unwrap();
        diffuse(humidity.0, humidity.1);
    }

    fn temperature_transfer(&self, temperature: &mut World2dField, seasons_phase: f64) {
        {
            let temperature = temperature.iterate().unwrap();
            diffuse(temperature.0, temperature.1);
        }
        {
            let temperature = temperature.iterate().unwrap();
            let rows = temperature.0.rows() as f64;
            transfer(
                temperature.0,
                temperature.1,
                self.config.temperature_transfer_rate,
                |_, r, _| {
                    let f = (PI * (r as f64 / rows + seasons_phase)).sin();
                    let d = self.config.temperature_range.end - self.config.temperature_range.start;
                    self.config.temperature_range.start + d * f
                },
            );
        }
    }

    fn rainfall_and_evaporation(
        &self,
        surface_water: &mut World2dField,
        humidity: &mut World2dField,
        temperature: &Grid2d<f64>,
    ) {
        if self.config.water_capacity <= 0.0 {
            return;
        }
        let surface_water = surface_water.iterate().unwrap();
        let humidity = humidity.iterate().unwrap();
        let it = surface_water
            .0
            .iter()
            .zip(surface_water.1.iter_mut())
            .zip(humidity.0.iter())
            .zip(humidity.1.iter_mut())
            .zip(temperature.iter());
        for ((((swp, swn), hp), hn), t) in it {
            let limit = remap_in_ranges(
                *t,
                self.config.temperature_range.clone(),
                self.config.humidity_limit_range.clone(),
            );
            let h = *hp - limit;
            let h = if h > 0.0 {
                h * self.config.rainfall_rate
            } else {
                h * self.config.evaporation_rate
            };
            let w = h * self.config.water_capacity;
            let w = if w < 0.0 { w.max(-swp) } else { w };
            let h = w / self.config.water_capacity;
            *hn = hp - h;
            *swn = swp + h * self.config.water_capacity;
        }
    }
}

impl World2dSimulation for World2dClimateSimulation {
    fn process_world(
        &mut self,
        altitude: &mut World2dField,
        temperature: &mut World2dField,
        humidity: &mut World2dField,
        surface_water: &mut World2dField,
    ) {
        let steps = self.steps + 1;
        if steps >= self.config.full_year_steps {
            self.years += 1;
            self.steps = 0;
        } else {
            self.steps = steps;
        }
        let seasons_phase = ((self.steps as f64 / self.config.full_year_steps as f64) * PI * 2.0)
            .sin()
            * self.config.world_axis_angle.sin();
        self.surface_water_transfer(altitude.get().unwrap(), surface_water);
        self.rainfall_and_evaporation(surface_water, humidity, temperature.get().unwrap());
        self.humidity_transfer(humidity);
        self.temperature_transfer(temperature, seasons_phase);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
