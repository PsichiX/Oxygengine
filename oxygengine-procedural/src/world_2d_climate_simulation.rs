use crate::world_2d::{World2dField, World2dSimulation};
use oxygengine_utils::grid_2d::Grid2d;
use psyche_utils::switch::Switch;
use std::{
    any::Any,
    f64::consts::{E, PI},
    ops::{Add, Div, Mul, Neg, Range, Sub},
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct World2dClimateSimulationVector(pub f64, pub f64);

impl World2dClimateSimulationVector {
    #[inline]
    pub fn neg_x(&self) -> Self {
        Self(-self.0, self.1)
    }

    #[inline]
    pub fn neg_y(&self) -> Self {
        Self(self.0, -self.1)
    }

    #[inline]
    pub fn dot(&self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }

    #[inline]
    pub fn refract(&self, normal: Self) -> Self {
        let len = self.magnitude();
        let dot = self.dot(normal);
        let offset = if dot >= 0.0 {
            normal * -dot * 2.0
        } else {
            normal * dot * 2.0
        };
        (*self + offset).normalized() * len
    }

    #[inline]
    pub fn magnitude_sqr(&self) -> f64 {
        self.0 * self.0 + self.1 * self.1
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_sqr().sqrt()
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self(self.0 / mag, self.1 / mag)
        } else {
            Self(0.0, 0.0)
        }
    }
}

impl From<(f64, f64)> for World2dClimateSimulationVector {
    fn from((x, y): (f64, f64)) -> Self {
        Self(x, y)
    }
}

impl Into<(f64, f64)> for World2dClimateSimulationVector {
    fn into(self) -> (f64, f64) {
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

impl Add<f64> for World2dClimateSimulationVector {
    type Output = Self;

    fn add(self, other: f64) -> Self {
        Self(self.0 + other, self.1 + other)
    }
}

impl Sub for World2dClimateSimulationVector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl Sub<f64> for World2dClimateSimulationVector {
    type Output = Self;

    fn sub(self, other: f64) -> Self {
        Self(self.0 - other, self.1 - other)
    }
}

impl Mul for World2dClimateSimulationVector {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0, self.1 * other.1)
    }
}

impl Mul<f64> for World2dClimateSimulationVector {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self(self.0 * other, self.1 * other)
    }
}

impl Div for World2dClimateSimulationVector {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0, self.1 / other.1)
    }
}

impl Div<f64> for World2dClimateSimulationVector {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Self(self.0 / other, self.1 / other)
    }
}

#[derive(Debug, Clone)]
pub struct World2dClimateSimulationConfig {
    pub world_axis_angle: f64,
    pub full_year_steps: usize,
    pub mass_diffuse_factor: f64,
    pub mass_diffuse_iterations: usize,
    pub mass_advect_factor: f64,
    pub viscosity_factor: f64,
    pub viscosity_iterations: usize,
    pub poisson_pressure_iterations: usize,
    pub slopeness_refraction_power: f64,
    pub water_capacity: f64,
    pub altitude_range: Range<f64>,
    pub temperature_range: Range<f64>,
    pub humidity_limit_range: Range<f64>,
    pub rainfall_factor: f64,
    pub evaporation_factor: f64,
    pub currents_flow_gain_factor: f64,
    pub coriolis_velocity_range: Range<f64>,
    pub coriolis_factor: f64,
    pub world_core_heating: f64,
    pub sun_heating: f64,
    pub sun_heating_adaptive_correction_factor: f64,
    pub sun_heating_absorption_surface_water_range: Range<f64>,
    pub thermal_radiation: f64,
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
            currents_flow_gain_factor: 1.0,
            coriolis_velocity_range: 1.0..5.0,
            coriolis_factor: 0.01,
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
    divergence: Option<Grid2d<f64>>,
    pressure: Option<Switch<Grid2d<f64>>>,
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

    pub fn divergence(&self) -> Option<&Grid2d<f64>> {
        if let Some(divergence) = &self.divergence {
            Some(divergence)
        } else {
            None
        }
    }

    pub fn pressure(&self) -> Option<&Grid2d<f64>> {
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
        surface_water: &Grid2d<f64>,
        seasons_phase: f64,
    ) {
        let temperature = temperature.get_mut().unwrap();
        let rows = temperature.rows() as f64;
        let cols = temperature.cols() as f64;
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
            let f = (PI * ((row as f64 + 0.5) / rows + seasons_phase)).sin();
            let sun_value = (sun_heating * f * absorption).max(0.0);
            value + world_core_heating + sun_value - thermal_radiation
        });
        let size = cols * rows;
        let average_temp = temperature.iter().fold(0.0, |a, v| a + *v) / size;
        // let dtemp = (target_average_temp - average_temp) + thermal_radiation - world_core_heating;
        let dtemp = target_average_temp - average_temp;
        // let dtemp = (logistic_sigmoid(dtemp.abs()) - 0.5) * 2.0 * dtemp;
        // let dtemp = (logistic_sigmoid(dtemp) - 0.5) * 2.0;
        let f = self.config.sun_heating_adaptive_correction_factor;
        self.config.sun_heating = (self.config.sun_heating + dtemp * f).max(0.0);
    }

    // fn process_currents(&mut self, temperature: &Grid2d<f64>) {
    //     if let Some(velocity) = &mut self.velocity {
    //         let velocity = velocity.iterate().unwrap();
    //         let velocity_prev = velocity.0;
    //         let velocity_next = velocity.1;
    //         let cols = velocity_prev.cols();
    //         let rows = velocity_prev.rows();
    //         let gain_factor = self.config.currents_flow_gain_factor;
    //         velocity_next.with(|col, row, _| {
    //             if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
    //                 (0.0, 0.0).into()
    //             } else {
    //                 let vel = velocity_prev[(col, row)];
    //                 let left = temperature[(col - 1, row)];
    //                 let right = temperature[(col + 1, row)];
    //                 let top = temperature[(col, row - 1)];
    //                 let bottom = temperature[(col, row + 1)];
    //                 let dx = right - left;
    //                 let dy = bottom - top;
    //                 // let dx = logistic_sigmoid_advanced_signed(dx, 1.0, 0.0, 0.1);
    //                 // let dy = logistic_sigmoid_advanced_signed(dy, 1.0, 0.0, 0.1);
    //                 let dx = if dx.abs() > 5.0 { dx } else { 0.0 };
    //                 let dy = if dy.abs() > 5.0 { dy } else { 0.0 };
    //                 let x = vel.0 - dx * gain_factor;
    //                 let y = vel.1 - dy * gain_factor;
    //                 (x, y).into()
    //             }
    //         });
    //         apply_mirror_boundaries(velocity_next);
    //     }
    // }

    // fn coriolis_effect(&mut self) {
    //     if let Some(velocity) = &mut self.velocity {
    //         let velocity = velocity.get_mut().unwrap();
    //         let rows = velocity.rows() as f64;
    //         let velocity_range = self.config.coriolis_velocity_range.clone();
    //         let factor = self.config.coriolis_factor;
    //         let diff = velocity_range.end - velocity_range.start;
    //         velocity.with(|col, row, vel| {
    //             let f = 1.0 - (PI * ((row as f64 + 0.5) / rows)).sin().abs();
    //             (vel.0 + (velocity_range.start + diff * f) * factor, vel.1).into()
    //         });
    //     }
    // }

    fn surface_water_transfer(&self, altitude: &Grid2d<f64>, surface_water: &mut World2dField) {
        let surface_water = surface_water.iterate().unwrap();
        diffuse_with_barriers(altitude, surface_water.0, surface_water.1);
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
}

impl World2dSimulation for World2dClimateSimulation {
    fn initialize_world(
        &mut self,
        altitude: &mut Grid2d<f64>,
        temperature: &mut Grid2d<f64>,
        _humidity: &mut Grid2d<f64>,
        _surface_water: &mut Grid2d<f64>,
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
            let f = (PI * ((row as f64 + 0.5) / rows as f64)).sin();
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
        let seasons_phase = ((self.steps as f64 / self.config.full_year_steps as f64) * PI * 2.0)
            .sin()
            * self.config.world_axis_angle.sin();

        self.heat_exchange(temperature, surface_water.get().unwrap(), seasons_phase);
        // self.coriolis_effect();
        // self.process_currents(temperature.get().unwrap());
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
                        // TODO: remove.
                        // {
                        //     let velocity = velocity.get_mut().unwrap();
                        //     velocity[(19, 19)] = World2dClimateSimulationVector(5.0, 0.0);
                        //     // velocity[(49, 50)] = World2dClimateSimulationVector(15.0, 0.0);
                        //     // velocity[(89, 9)] = World2dClimateSimulationVector(-20.0, 0.0);
                        //     // velocity[(89, 89)] = World2dClimateSimulationVector(-6.0, 0.0);
                        //     // for row in 30..70 {
                        //     //     velocity[(45, row)] = World2dClimateSimulationVector(5.0, 0.0);
                        //     // }
                        //     // for row in 45..55 {
                        //     //     velocity[(55, row)] = World2dClimateSimulationVector(-5.0, 0.0);
                        //     // }
                        // }
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

fn apply_duplicate_boundaries(field: &mut Grid2d<f64>) {
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
    refraction_power: f64,
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
fn diffuse_scalar(field: &mut World2dField, iterations: usize, factor: f64) {
    let cols = field.get().unwrap().cols();
    let rows = field.get().unwrap().rows();
    let fa = (cols as f64 * rows as f64) * factor;
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
    factor: f64,
) {
    let cols = field.get().unwrap().cols();
    let rows = field.get().unwrap().rows();
    let fa = (cols as f64 * rows as f64) * factor;
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
    density_prev: &Grid2d<f64>,
    density_next: &mut Grid2d<f64>,
    velocity: &Grid2d<World2dClimateSimulationVector>,
    factor: f64,
) {
    let cols = density_prev.cols();
    let rows = density_prev.rows();
    density_next.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            0.0
        } else {
            let vel = velocity[(col, row)];
            let x = (col as f64 - factor * vel.0)
                .min(cols as f64 - 1.5)
                .max(0.5);
            let y = (row as f64 - factor * vel.1)
                .min(rows as f64 - 1.5)
                .max(0.5);
            let xa = x as usize;
            let ya = y as usize;
            let xb = xa + 1;
            let yb = ya + 1;
            let dx1 = x - xa as f64;
            let dx0 = 1.0 - dx1;
            let dy1 = y - ya as f64;
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
    factor: f64,
) {
    let cols = field_prev.cols();
    let rows = field_prev.rows();
    field_next.with(|col, row, _| {
        if col == 0 || col == cols - 1 || row == 0 || row == rows - 1 {
            (0.0, 0.0).into()
        } else {
            let vel = velocity[(col, row)];
            let x = (col as f64 - factor * vel.0)
                .min(cols as f64 - 1.5)
                .max(0.5);
            let y = (row as f64 - factor * vel.1)
                .min(rows as f64 - 1.5)
                .max(0.5);
            let xa = x as usize;
            let ya = y as usize;
            let xb = xa + 1;
            let yb = ya + 1;
            let dx1 = x - xa as f64;
            let dx0 = 1.0 - dx1;
            let dy1 = y - ya as f64;
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
    divergence: &mut Grid2d<f64>,
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
    divergence: &mut Grid2d<f64>,
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
    pressure: &Grid2d<f64>,
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
    barriers: &Grid2d<f64>,
    field_prev: &Grid2d<f64>,
    field_next: &mut Grid2d<f64>,
) {
    let levels = (barriers + field_prev).unwrap();
    field_next.with(|col, row, _| {
        let sample_coord = (if col == 0 { 0 } else { 1 }, if row == 0 { 0 } else { 1 });
        let barriers_sample = barriers.sample((col, row), 1);
        let values_sample = field_prev.sample((col, row), 1);
        let levels_sample = levels.sample((col, row), 1);
        let levels_min = *levels_sample
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let levels_max = *levels_sample
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let capacity_sample = barriers_sample.map(|_, _, v| levels_max - v.max(levels_min));
        let capacity = capacity_sample.iter().fold(0.0, |a, v| a + v);
        if capacity > 0.0 {
            let energy_sample = Grid2d::from((
                levels_sample.cols(),
                levels_sample
                    .iter()
                    .zip(barriers_sample.iter())
                    .map(|(v, b)| *v - b.max(levels_min)),
            ));
            let energy = energy_sample.iter().fold(0.0, |a, v| a + v);
            let amount = energy * capacity_sample[sample_coord] / capacity;
            values_sample[sample_coord] - energy_sample[sample_coord] + amount
        } else {
            values_sample[sample_coord]
        }
    });
    // error correction.
    let before = field_prev.iter().fold(0.0, |a, v| a + v);
    let after = field_next.iter().fold(0.0, |a, v| a + v);
    let diff = (before - after) / field_prev.len() as f64;
    field_next.with(|_, _, value| *value + diff);
}

#[inline]
fn unproject_in_range(value: f64, range: Range<f64>) -> f64 {
    (value - range.start) / (range.end - range.start)
}

#[inline]
fn project_in_range(factor: f64, range: Range<f64>) -> f64 {
    (range.end - range.start) * factor + range.start
}

#[inline]
fn remap_in_ranges(value: f64, from: Range<f64>, to: Range<f64>) -> f64 {
    project_in_range(unproject_in_range(value, from).max(0.0).min(1.0), to)
}

#[inline]
fn logistic_sigmoid_simple(value: f64) -> f64 {
    logistic_sigmoid_advanced(value, 1.0, 0.0, 1.0)
}

#[inline]
fn logistic_sigmoid_simple_signed(value: f64) -> f64 {
    logistic_sigmoid_advanced_signed(value, 1.0, 0.0, 1.0)
}

#[inline]
fn logistic_sigmoid_advanced(
    value: f64,
    curve_max: f64,
    midpoint: f64,
    logistic_growth: f64,
) -> f64 {
    let power = -logistic_growth * (value - midpoint);
    let denominator = 1.0 + E.powf(power);
    curve_max / denominator
}

#[inline]
fn logistic_sigmoid_advanced_signed(
    value: f64,
    curve_max: f64,
    midpoint: f64,
    logistic_growth: f64,
) -> f64 {
    (logistic_sigmoid_advanced(value.abs(), curve_max, midpoint, logistic_growth) - 0.5)
        * 2.0
        * value
}
