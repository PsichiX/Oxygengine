#![allow(dead_code)]

use crate::world_2d::{World2dField, World2dSimulation};
use oxygengine_utils::{
    grid_2d::{Grid2d, Grid2dNeighborSample},
    Scalar,
};
use psyche_utils::switch::Switch;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "scalar64"))]
use std::f32::consts::{E, PI};
#[cfg(feature = "scalar64")]
use std::f64::consts::{E, PI};
use std::{
    any::Any,
    ops::{Add, Div, Mul, Neg, Range, Sub},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct World2dClimateSimulationVector(pub Scalar, pub Scalar);

impl World2dClimateSimulationVector {
    #[inline]
    pub fn neg_x(self) -> Self {
        Self(-self.0, self.1)
    }

    #[inline]
    pub fn neg_y(self) -> Self {
        Self(self.0, -self.1)
    }

    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.0 * other.0 + self.1 * other.1
    }

    #[inline]
    pub fn refract(self, normal: Self) -> Self {
        let len = self.magnitude();
        let dot = self.dot(normal);
        let offset = if dot >= 0.0 {
            normal * -dot * 2.0
        } else {
            normal * dot * 2.0
        };
        (self + offset).normalized() * len
    }

    #[inline]
    pub fn magnitude_sqr(self) -> Scalar {
        self.0 * self.0 + self.1 * self.1
    }

    #[inline]
    pub fn magnitude(self) -> Scalar {
        self.magnitude_sqr().sqrt()
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self(self.0 / mag, self.1 / mag)
        } else {
            Self(0.0, 0.0)
        }
    }
}

impl From<(Scalar, Scalar)> for World2dClimateSimulationVector {
    fn from((x, y): (Scalar, Scalar)) -> Self {
        Self(x, y)
    }
}

impl Into<(Scalar, Scalar)> for World2dClimateSimulationVector {
    fn into(self) -> (Scalar, Scalar) {
        (self.0, self.1)
    }
}

impl Neg for World2dClimateSimulationVector {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0, -self.1)
    }
}

impl Add for World2dClimateSimulationVector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl Add<Scalar> for World2dClimateSimulationVector {
    type Output = Self;

    fn add(self, other: Scalar) -> Self {
        Self(self.0 + other, self.1 + other)
    }
}

impl Sub for World2dClimateSimulationVector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl Sub<Scalar> for World2dClimateSimulationVector {
    type Output = Self;

    fn sub(self, other: Scalar) -> Self {
        Self(self.0 - other, self.1 - other)
    }
}

impl Mul for World2dClimateSimulationVector {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0, self.1 * other.1)
    }
}

impl Mul<Scalar> for World2dClimateSimulationVector {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Self(self.0 * other, self.1 * other)
    }
}

impl Div for World2dClimateSimulationVector {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0, self.1 / other.1)
    }
}

impl Div<Scalar> for World2dClimateSimulationVector {
    type Output = Self;

    fn div(self, other: Scalar) -> Self {
        Self(self.0 / other, self.1 / other)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World2dClimateSimulationConfig {
    pub world_axis_angle: Scalar,
    pub full_year_steps: usize,
    pub mass_diffuse_factor: Scalar,
    pub mass_diffuse_iterations: usize,
    pub mass_advect_factor: Scalar,
    pub viscosity_factor: Scalar,
    pub viscosity_iterations: usize,
    pub poisson_pressure_iterations: usize,
    pub slopeness_refraction_power: Scalar,
    pub water_capacity: Scalar,
    pub altitude_range: Range<Scalar>,
    pub temperature_range: Range<Scalar>,
    pub humidity_limit_range: Range<Scalar>,
    pub rainfall_factor: Scalar,
    pub evaporation_factor: Scalar,
    pub world_core_heating: Scalar,
    pub sun_heating: Scalar,
    pub sun_heating_adaptive_correction_factor: Scalar,
    pub sun_heating_absorption_surface_water_range: Range<Scalar>,
    pub thermal_radiation: Scalar,
}

impl Default for World2dClimateSimulationConfig {
    fn default() -> Self {
        Self {
            world_axis_angle: 5.0 * PI / 180.0,
            full_year_steps: 364,
            mass_diffuse_factor: 0.0001,
            mass_diffuse_iterations: 5,
            mass_advect_factor: 1.0,
            viscosity_factor: 0.0001,
            viscosity_iterations: 5,
            poisson_pressure_iterations: 5,
            slopeness_refraction_power: 3.0,
            water_capacity: 100.0,
            altitude_range: 0.0..100.0,
            temperature_range: 0.0..100.0,
            humidity_limit_range: 0.05..0.2,
            rainfall_factor: 0.1,
            evaporation_factor: 0.05,
            world_core_heating: 1.0,
            sun_heating: 0.0,
            sun_heating_adaptive_correction_factor: 1.0,
            sun_heating_absorption_surface_water_range: 1.0..0.01,
            thermal_radiation: 1.0,
        }
    }
}

#[derive(Default)]
pub struct World2dClimateSimulation {
    config: World2dClimateSimulationConfig,
    steps: usize,
    years: usize,
    velocity: Option<Switch<Grid2d<World2dClimateSimulationVector>>>,
    divergence: Option<Grid2d<Scalar>>,
    pressure: Option<Switch<Grid2d<Scalar>>>,
    slopeness: Option<Grid2d<World2dClimateSimulationVector>>,
}

impl World2dClimateSimulation {
    pub fn new(config: World2dClimateSimulationConfig) -> Self {
        Self {
            config,
            steps: 0,
            years: 0,
            velocity: None,
            divergence: None,
            pressure: None,
            slopeness: None,
        }
    }

    pub fn config(&self) -> &World2dClimateSimulationConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut World2dClimateSimulationConfig {
        &mut self.config
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn years(&self) -> usize {
        self.years
    }

    pub fn velocity(&self) -> Option<&Grid2d<World2dClimateSimulationVector>> {
        if let Some(velocity) = &self.velocity {
            velocity.get()
        } else {
            None
        }
    }

    pub fn divergence(&self) -> Option<&Grid2d<Scalar>> {
        if let Some(divergence) = &self.divergence {
            Some(divergence)
        } else {
            None
        }
    }

    pub fn pressure(&self) -> Option<&Grid2d<Scalar>> {
        if let Some(pressure) = &self.pressure {
            pressure.get()
        } else {
            None
        }
    }

    pub fn slopeness(&self) -> Option<&Grid2d<World2dClimateSimulationVector>> {
        if let Some(slopeness) = &self.slopeness {
            Some(slopeness)
        } else {
            None
        }
    }

    pub fn rebuild_slopeness(&mut self) {
        self.slopeness = None;
    }

    fn heat_exchange(
        &mut self,
        temperature: &mut World2dField,
        surface_water: &Grid2d<Scalar>,
        seasons_phase: Scalar,
    ) {
        let temperature = temperature.get_mut().unwrap();
        let rows = temperature.rows() as Scalar;
        let cols = temperature.cols() as Scalar;
        let world_core_heating = self.config.world_core_heating.max(0.0);
        let thermal_radiation = self.config.thermal_radiation.max(0.0);
        let sun_heating = self.config.sun_heating.max(0.0);
        let absorption_diff = self.config.sun_heating_absorption_surface_water_range.end
            - self.config.sun_heating_absorption_surface_water_range.start;
        let target_average_temp =
            (self.config.temperature_range.start + self.config.temperature_range.end) * 0.5;
        temperature.with(|col, row, value| {
            let water = surface_water[(col, row)];
            let f = if self.config.water_capacity > 0.0 {
                (water / self.config.water_capacity).max(0.0).min(1.0)
            } else {
                0.0
            };
            let absorption =
                self.config.sun_heating_absorption_surface_water_range.start + absorption_diff * f;
            let f = (PI * ((row as Scalar + 0.5) / rows + seasons_phase)).sin();
            let sun_value = (sun_heating * f * absorption).max(0.0);
            value + world_core_heating + sun_value - thermal_radiation
        });
        let size = cols * rows;
        #[cfg(not(feature = "parallel"))]
        let average_temp = temperature.iter().sum::<Scalar>() / size;
        #[cfg(feature = "parallel")]
        let average_temp = temperature.par_iter().sum::<Scalar>() / size;
        let dtemp = logistic_sigmoid_simple_signed(target_average_temp - average_temp);
        let f = self.config.sun_heating_adaptive_correction_factor;
        self.config.sun_heating = (self.config.sun_heating + dtemp * f).max(0.0);
    }

    fn surface_water_transfer(&self, altitude: &Grid2d<Scalar>, surface_water: &mut World2dField) {
        let surface_water = surface_water.iterate().unwrap();
        diffuse_with_barriers(altitude, surface_water.0, surface_water.1);
    }

    fn rainfall_and_evaporation(
        &self,
        surface_water: &mut World2dField,
        humidity: &mut World2dField,
        temperature: &Grid2d<Scalar>,
    ) {
        if self.config.water_capacity <= 0.0 {
            return;
        }
        let surface_water = surface_water.iterate().unwrap();
        let humidity = humidity.iterate().unwrap();
        #[cfg(not(feature = "parallel"))]
        {
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
                    h * self.config.rainfall_factor
                } else {
                    h * self.config.evaporation_factor
                };
                let w = (h * self.config.water_capacity).max(-swp);
                let h = w / self.config.water_capacity;
                *hn = hp - h;
                *swn = swp + w;
            }
        }
        #[cfg(feature = "parallel")]
        {
            surface_water
                .0
                .par_iter()
                .zip(surface_water.1.par_iter_mut())
                .zip(humidity.0.par_iter())
                .zip(humidity.1.par_iter_mut())
                .zip(temperature.par_iter())
                .for_each(|((((swp, swn), hp), hn), t)| {
                    let limit = remap_in_ranges(
                        *t,
                        self.config.temperature_range.clone(),
                        self.config.humidity_limit_range.clone(),
                    );
                    let h = *hp - limit;
                    let h = if h > 0.0 {
                        h * self.config.rainfall_factor
                    } else {
                        h * self.config.evaporation_factor
                    };
                    let w = (h * self.config.water_capacity).max(-swp);
                    let h = w / self.config.water_capacity;
                    *hn = hp - h;
                    *swn = swp + w;
                });
        }
    }
}

impl World2dSimulation for World2dClimateSimulation {
    fn initialize_world(
        &mut self,
        altitude: &mut Grid2d<Scalar>,
        temperature: &mut Grid2d<Scalar>,
        _humidity: &mut Grid2d<Scalar>,
        _surface_water: &mut Grid2d<Scalar>,
    ) {
        self.velocity = Some(Switch::new(
            2,
            Grid2d::new(altitude.cols(), altitude.rows(), (0.0, 0.0).into()),
        ));
        self.divergence = Some(Grid2d::new(altitude.cols(), altitude.rows(), 0.0));
        self.pressure = Some(Switch::new(
            2,
            Grid2d::new(altitude.cols(), altitude.rows(), 0.0),
        ));
        let cols = altitude.cols();
        let rows = altitude.rows();
        let diff = self.config.altitude_range.end - self.config.altitude_range.start;
        let mut slopeness = Grid2d::new(cols, rows, (0.0, 0.0).into());
        slopeness.with(|col, row, _| {
            if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
                (0.0, 0.0).into()
            } else {
                let left = altitude[(col - 1, row)];
                let right = altitude[(col + 1, row)];
                let top = altitude[(col, row - 1)];
                let bottom = altitude[(col, row + 1)];
                let dx = (right - left) / diff;
                let dy = (bottom - top) / diff;
                (dx, dy).into()
            }
        });
        self.slopeness = Some(slopeness);
        let diff = self.config.temperature_range.end - self.config.temperature_range.start;
        temperature.with(|_, row, _| {
            let f = (PI * ((row as Scalar + 0.5) / rows as Scalar)).sin();
            self.config.temperature_range.start + diff * f
        });
    }

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
        let seasons_phase =
            ((self.steps as Scalar / self.config.full_year_steps as Scalar) * PI * 2.0).sin()
                * self.config.world_axis_angle.sin();

        self.heat_exchange(temperature, surface_water.get().unwrap(), seasons_phase);
        self.surface_water_transfer(altitude.get().unwrap(), surface_water);
        self.rainfall_and_evaporation(surface_water, humidity, temperature.get().unwrap());

        // process temperature and humidity as density fields.
        {
            {
                diffuse_scalar(
                    temperature,
                    self.config.mass_diffuse_iterations,
                    self.config.mass_diffuse_factor,
                );
            }
            {
                diffuse_scalar(
                    humidity,
                    self.config.mass_diffuse_iterations,
                    self.config.mass_diffuse_factor,
                );
            }
            if let Some(velocity) = &self.velocity {
                let velocity = velocity.get().unwrap();
                {
                    let temperature = temperature.iterate().unwrap();
                    let temperature_prev = temperature.0;
                    let temperature_next = temperature.1;
                    advect_scalar(
                        temperature_prev,
                        temperature_next,
                        velocity,
                        self.config.mass_advect_factor,
                    );
                }
                {
                    let humidity = humidity.iterate().unwrap();
                    let humidity_prev = humidity.0;
                    let humidity_next = humidity.1;
                    advect_scalar(
                        humidity_prev,
                        humidity_next,
                        velocity,
                        self.config.mass_advect_factor,
                    );
                }
            }
        }
        // process velocity flow.
        {
            if let Some(velocity) = &mut self.velocity {
                if let Some(divergence) = &mut self.divergence {
                    if let Some(pressure) = &mut self.pressure {
                        // modify velocities by slopeness
                        if self.config.slopeness_refraction_power > 0.0 {
                            if let Some(slopeness) = &self.slopeness {
                                {
                                    let velocity = velocity.get_mut().unwrap();
                                    consider_obstacles(
                                        velocity,
                                        slopeness,
                                        self.config.slopeness_refraction_power,
                                    );
                                }
                                // TODO: test if it is needed.
                                conserve_mass(
                                    velocity,
                                    pressure,
                                    divergence,
                                    self.config.poisson_pressure_iterations,
                                );
                            }
                        }
                        // diffuse velocity
                        {
                            diffuse_vector(
                                velocity,
                                self.config.viscosity_iterations,
                                self.config.viscosity_factor,
                            );
                            conserve_mass(
                                velocity,
                                pressure,
                                divergence,
                                self.config.poisson_pressure_iterations,
                            );
                        }
                        // velocity flow
                        {
                            {
                                let velocity = velocity.iterate().unwrap();
                                let velocity_prev = velocity.0;
                                let velocity_next = velocity.1;
                                advect_vector(
                                    velocity_prev,
                                    velocity_next,
                                    velocity_prev,
                                    self.config.mass_advect_factor,
                                );
                            }
                            conserve_mass(
                                velocity,
                                pressure,
                                divergence,
                                self.config.poisson_pressure_iterations,
                            );
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<&World2dClimateSimulationData> for World2dClimateSimulation {
    fn from(data: &World2dClimateSimulationData) -> Self {
        Self {
            config: data.config.clone(),
            steps: data.steps,
            years: data.years,
            velocity: if let Some(ref velocity) = data.velocity {
                Some(Switch::new(2, velocity.clone()))
            } else {
                None
            },
            divergence: data.divergence.clone(),
            pressure: if let Some(ref pressure) = data.pressure {
                Some(Switch::new(2, pressure.clone()))
            } else {
                None
            },
            slopeness: data.slopeness.clone(),
        }
    }
}

fn apply_duplicate_boundaries(field: &mut Grid2d<Scalar>) {
    let cols = field.cols();
    let rows = field.rows();
    for col in 1..(cols - 1) {
        field[(col, 0)] = field[(col, 1)];
        field[(col, rows - 1)] = field[(col, rows - 2)];
    }
    for row in 1..(rows - 1) {
        field[(0, row)] = field[(1, row)];
        field[(cols - 1, row)] = field[(cols - 2, row)];
    }
    field[(0, 0)] = field[(1, 1)];
    field[(cols - 1, 0)] = field[(cols - 2, 1)];
    field[(cols - 1, rows - 1)] = field[(cols - 2, rows - 2)];
    field[(0, rows - 1)] = field[(1, rows - 2)];
}

fn apply_mirror_boundaries(field: &mut Grid2d<World2dClimateSimulationVector>) {
    let cols = field.cols();
    let rows = field.rows();
    for col in 1..(cols - 1) {
        field[(col, 0)] = field[(col, 1)].neg_y();
        field[(col, rows - 1)] = field[(col, rows - 2)].neg_y();
    }
    for row in 1..(rows - 1) {
        field[(0, row)] = field[(1, row)].neg_x();
        field[(cols - 1, row)] = field[(cols - 2, row)].neg_x();
    }
    field[(0, 0)] = (field[(0, 1)] + field[(1, 0)]) * 0.5;
    field[(cols - 1, 0)] = (field[(cols - 2, 0)] + field[(cols - 1, 1)]) * 0.5;
    field[(cols - 1, rows - 1)] = (field[(cols - 2, rows - 1)] + field[(cols - 1, rows - 2)]) * 0.5;
    field[(0, rows - 1)] = (field[(1, rows - 1)] + field[(0, rows - 2)]) * 0.5;
}

fn consider_obstacles(
    velocity: &mut Grid2d<World2dClimateSimulationVector>,
    slopeness: &Grid2d<World2dClimateSimulationVector>,
    refraction_power: Scalar,
) {
    let cols = velocity.cols();
    let rows = velocity.rows();
    velocity.with(|col, row, value| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            (0.0, 0.0).into()
        } else {
            let slope = slopeness[(col, row)];
            let normal = slope.normalized();
            let scale = 1.0 - slope.magnitude();
            let scale = 1.0 - scale.powf(refraction_power);
            value.refract(normal * scale)
        }
    });
    apply_mirror_boundaries(velocity);
}

// a.k.a. Jacobi
fn diffuse_scalar(field: &mut World2dField, iterations: usize, factor: Scalar) {
    let cols = field.get().unwrap().cols();
    let rows = field.get().unwrap().rows();
    let fa = (cols as Scalar * rows as Scalar) * factor;
    let fb = 1.0 / (1.0 + 4.0 * fa);
    for _ in 0..iterations {
        let field = field.iterate().unwrap();
        let field_prev = field.0;
        let field_next = field.1;
        field_next.with(|col, row, _| {
            if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
                0.0
            } else {
                let center = field_prev[(col, row)];
                let left = field_prev[(col - 1, row)];
                let right = field_prev[(col + 1, row)];
                let top = field_prev[(col, row - 1)];
                let bottom = field_prev[(col, row + 1)];
                ((left + right + top + bottom) * fa + center) * fb
            }
        });
        apply_duplicate_boundaries(field_next);
    }
}

// a.k.a. Jacobi
fn diffuse_vector(
    field: &mut Switch<Grid2d<World2dClimateSimulationVector>>,
    iterations: usize,
    factor: Scalar,
) {
    let cols = field.get().unwrap().cols();
    let rows = field.get().unwrap().rows();
    let fa = (cols as Scalar * rows as Scalar) * factor;
    let fb = 1.0 / (1.0 + 4.0 * fa);
    for _ in 0..iterations {
        let field = field.iterate().unwrap();
        let field_prev = field.0;
        let field_next = field.1;
        field_next.with(|col, row, _| {
            if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
                (0.0, 0.0).into()
            } else {
                let center = field_prev[(col, row)];
                let left = field_prev[(col - 1, row)];
                let right = field_prev[(col + 1, row)];
                let top = field_prev[(col, row - 1)];
                let bottom = field_prev[(col, row + 1)];
                ((left + right + top + bottom) * fa + center) * fb
            }
        });
        apply_mirror_boundaries(field_next);
    }
}

fn advect_scalar(
    density_prev: &Grid2d<Scalar>,
    density_next: &mut Grid2d<Scalar>,
    velocity: &Grid2d<World2dClimateSimulationVector>,
    factor: Scalar,
) {
    let cols = density_prev.cols();
    let rows = density_prev.rows();
    density_next.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            0.0
        } else {
            let vel = velocity[(col, row)];
            let x = (col as Scalar - factor * vel.0)
                .min(cols as Scalar - 1.5)
                .max(0.5);
            let y = (row as Scalar - factor * vel.1)
                .min(rows as Scalar - 1.5)
                .max(0.5);
            let xa = x as usize;
            let ya = y as usize;
            let xb = xa + 1;
            let yb = ya + 1;
            let dx1 = x - xa as Scalar;
            let dx0 = 1.0 - dx1;
            let dy1 = y - ya as Scalar;
            let dy0 = 1.0 - dy1;
            let val_xaya = density_prev[(xa, ya)];
            let val_xayb = density_prev[(xa, yb)];
            let val_xbya = density_prev[(xb, ya)];
            let val_xbyb = density_prev[(xb, yb)];
            (val_xaya * dy0 + val_xayb * dy1) * dx0 + (val_xbya * dy0 + val_xbyb * dy1) * dx1
        }
    });
    apply_duplicate_boundaries(density_next);
}

fn advect_vector(
    field_prev: &Grid2d<World2dClimateSimulationVector>,
    field_next: &mut Grid2d<World2dClimateSimulationVector>,
    velocity: &Grid2d<World2dClimateSimulationVector>,
    factor: Scalar,
) {
    let cols = field_prev.cols();
    let rows = field_prev.rows();
    field_next.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            (0.0, 0.0).into()
        } else {
            let vel = velocity[(col, row)];
            let x = (col as Scalar - factor * vel.0)
                .min(cols as Scalar - 1.5)
                .max(0.5);
            let y = (row as Scalar - factor * vel.1)
                .min(rows as Scalar - 1.5)
                .max(0.5);
            let xa = x as usize;
            let ya = y as usize;
            let xb = xa + 1;
            let yb = ya + 1;
            let dx1 = x - xa as Scalar;
            let dx0 = 1.0 - dx1;
            let dy1 = y - ya as Scalar;
            let dy0 = 1.0 - dy1;
            let val_xaya = field_prev[(xa, ya)];
            let val_xayb = field_prev[(xa, yb)];
            let val_xbya = field_prev[(xb, ya)];
            let val_xbyb = field_prev[(xb, yb)];
            (val_xaya * dy0 + val_xayb * dy1) * dx0 + (val_xbya * dy0 + val_xbyb * dy1) * dx1
        }
    });
    apply_mirror_boundaries(field_next);
}

fn conserve_mass(
    velocity: &mut Switch<Grid2d<World2dClimateSimulationVector>>,
    pressure: &mut World2dField,
    divergence: &mut Grid2d<Scalar>,
    poisson_pressure_iterations: usize,
) {
    {
        let velocity = velocity.get().unwrap();
        calculate_poisson_pressure(velocity, pressure, divergence, poisson_pressure_iterations);
    }
    {
        let velocity = velocity.iterate().unwrap();
        let velocity_prev = velocity.0;
        let velocity_next = velocity.1;
        let pressure = pressure.get().unwrap();
        calculate_convergence_from_pressure_gradient(velocity_prev, velocity_next, pressure);
    }
}

fn calculate_poisson_pressure(
    velocity: &Grid2d<World2dClimateSimulationVector>,
    pressure: &mut World2dField,
    divergence: &mut Grid2d<Scalar>,
    iterations: usize,
) {
    let cols = divergence.cols();
    let rows = divergence.rows();
    divergence.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            0.0
        } else {
            let left = velocity[(col - 1, row)].0;
            let right = velocity[(col + 1, row)].0;
            let top = velocity[(col, row - 1)].1;
            let bottom = velocity[(col, row + 1)].1;
            -0.5 * (right - left + bottom - top)
        }
    });
    apply_duplicate_boundaries(divergence);
    pressure.get_mut().unwrap().with(|_, _, _| 0.0);
    for _ in 0..iterations {
        let pressure = pressure.iterate().unwrap();
        let pressure_prev = pressure.0;
        let pressure_next = pressure.1;
        pressure_next.with(|col, row, _| {
            if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
                0.0
            } else {
                let div = divergence[(col, row)];
                let left = pressure_prev[(col - 1, row)];
                let right = pressure_prev[(col + 1, row)];
                let top = pressure_prev[(col, row - 1)];
                let bottom = pressure_prev[(col, row + 1)];
                (div + left + right + top + bottom) * 0.25
            }
        });
        apply_duplicate_boundaries(pressure_next);
    }
}

fn calculate_convergence_from_pressure_gradient(
    velocity_prev: &Grid2d<World2dClimateSimulationVector>,
    velocity_next: &mut Grid2d<World2dClimateSimulationVector>,
    pressure: &Grid2d<Scalar>,
) {
    let cols = velocity_prev.cols();
    let rows = velocity_prev.rows();
    velocity_next.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            (0.0, 0.0).into()
        } else {
            let left = pressure[(col - 1, row)];
            let right = pressure[(col + 1, row)];
            let top = pressure[(col, row - 1)];
            let bottom = pressure[(col, row + 1)];
            let vel = velocity_prev[(col, row)];
            let x = vel.0 - 0.5 * (right - left);
            let y = vel.1 - 0.5 * (bottom - top);
            (x, y).into()
        }
    });
    apply_mirror_boundaries(velocity_next);
}

fn diffuse_with_barriers(
    barriers: &Grid2d<Scalar>,
    field_prev: &Grid2d<Scalar>,
    field_next: &mut Grid2d<Scalar>,
) {
    let levels = (barriers + field_prev).unwrap();
    field_next.with(|col, row, _| {
        let sample_coord = (if col == 0 { 0 } else { 1 }, if row == 0 { 0 } else { 1 });
        let barriers_sample = barriers.neighbor_sample((col, row));
        let values_sample = field_prev.neighbor_sample((col, row));
        let levels_sample = levels.neighbor_sample((col, row));
        let levels_min = *levels_sample
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let levels_max = *levels_sample
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let capacity_sample = barriers_sample.map(|_, _, v| levels_max - v.max(levels_min));
        let capacity = capacity_sample.iter().sum::<Scalar>();
        if capacity > 0.0 {
            let energy_sample = Grid2dNeighborSample::from((
                levels_sample.cols(),
                levels_sample
                    .iter()
                    .zip(barriers_sample.iter())
                    .map(|(v, b)| *v - b.max(levels_min)),
            ));
            let energy = energy_sample.iter().sum::<Scalar>();
            let amount = energy * capacity_sample[sample_coord] / capacity;
            values_sample[sample_coord] - energy_sample[sample_coord] + amount
        } else {
            values_sample[sample_coord]
        }
    });
    // error correction.
    #[cfg(not(feature = "parallel"))]
    let before = field_prev.iter().sum::<Scalar>();
    #[cfg(feature = "parallel")]
    let before = field_prev.par_iter().sum::<Scalar>();
    #[cfg(not(feature = "parallel"))]
    let after = field_next.iter().sum::<Scalar>();
    #[cfg(feature = "parallel")]
    let after = field_next.par_iter().sum::<Scalar>();
    let diff = (before - after) / field_prev.len() as Scalar;
    field_next.with(|_, _, value| *value + diff);
}

#[inline]
fn unproject_in_range(value: Scalar, range: Range<Scalar>) -> Scalar {
    (value - range.start) / (range.end - range.start)
}

#[inline]
fn project_in_range(factor: Scalar, range: Range<Scalar>) -> Scalar {
    (range.end - range.start) * factor + range.start
}

#[inline]
fn remap_in_ranges(value: Scalar, from: Range<Scalar>, to: Range<Scalar>) -> Scalar {
    project_in_range(unproject_in_range(value, from).max(0.0).min(1.0), to)
}

#[inline]
fn logistic_sigmoid_simple(value: Scalar) -> Scalar {
    logistic_sigmoid_advanced(value, 1.0, 0.0, 1.0)
}

#[inline]
fn logistic_sigmoid_simple_signed(value: Scalar) -> Scalar {
    logistic_sigmoid_advanced_signed(value, 1.0, 0.0, 1.0)
}

#[inline]
fn logistic_sigmoid_advanced(
    value: Scalar,
    curve_max: Scalar,
    midpoint: Scalar,
    logistic_growth: Scalar,
) -> Scalar {
    let power = -logistic_growth * (value - midpoint);
    let denominator = 1.0 + E.powf(power);
    curve_max / denominator
}

#[inline]
fn logistic_sigmoid_advanced_signed(
    value: Scalar,
    curve_max: Scalar,
    midpoint: Scalar,
    logistic_growth: Scalar,
) -> Scalar {
    (logistic_sigmoid_advanced(value.abs(), curve_max, midpoint, logistic_growth) - 0.5)
        * 2.0
        * value
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World2dClimateSimulationData {
    config: World2dClimateSimulationConfig,
    steps: usize,
    years: usize,
    velocity: Option<Grid2d<World2dClimateSimulationVector>>,
    divergence: Option<Grid2d<Scalar>>,
    pressure: Option<Grid2d<Scalar>>,
    slopeness: Option<Grid2d<World2dClimateSimulationVector>>,
}

impl From<&World2dClimateSimulation> for World2dClimateSimulationData {
    fn from(sim: &World2dClimateSimulation) -> Self {
        Self {
            config: sim.config.clone(),
            steps: sim.steps,
            years: sim.years,
            velocity: if let Some(ref velocity) = sim.velocity {
                Some(velocity.get().unwrap().clone())
            } else {
                None
            },
            divergence: sim.divergence.clone(),
            pressure: if let Some(ref pressure) = sim.pressure {
                Some(pressure.get().unwrap().clone())
            } else {
                None
            },
            slopeness: sim.slopeness.clone(),
        }
    }
}
