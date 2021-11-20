use oxygengine_utils::{grid_2d::Grid2d, noise_map_generator::NoiseMapGenerator, Scalar};
use psyche_utils::switch::Switch;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "scalar64"))]
use std::f32::{INFINITY as SCALAR_INFINITY, NEG_INFINITY as SCALAR_NEG_INFINITY};
#[cfg(feature = "scalar64")]
use std::f64::{INFINITY as SCALAR_INFINITY, NEG_INFINITY as SCALAR_NEG_INFINITY};
use std::{any::Any, borrow::Borrow, ops::Range};

pub type World2dField = Switch<Grid2d<Scalar>>;

#[derive(Debug, Clone)]
pub struct World2dConfig {
    pub size: usize,
    pub zoom: Scalar,
    pub altitude_seed: u32,
    pub altitude_range: Range<Scalar>,
    pub temperature_seed: u32,
    pub temperature_range: Range<Scalar>,
    pub humidity_seed: u32,
    pub humidity_range: Range<Scalar>,
}

impl Default for World2dConfig {
    fn default() -> Self {
        Self {
            size: 100,
            zoom: 5.0,
            altitude_seed: 1,
            altitude_range: 0.0..100.0,
            temperature_seed: 2,
            temperature_range: 0.0..100.0,
            humidity_seed: 3,
            humidity_range: 0.1..1.0,
        }
    }
}

pub trait World2dSimulation: Any + Send + Sync {
    fn initialize_world(
        &mut self,
        _altitude: &mut Grid2d<Scalar>,
        _temperature: &mut Grid2d<Scalar>,
        _humidity: &mut Grid2d<Scalar>,
        _surface_water: &mut Grid2d<Scalar>,
    ) {
    }

    fn process_world(
        &mut self,
        altitude: &mut World2dField,
        temperature: &mut World2dField,
        humidity: &mut World2dField,
        surface_water: &mut World2dField,
    );

    fn as_any(&self) -> &dyn Any;
}

impl World2dSimulation for () {
    fn process_world(
        &mut self,
        _altitude: &mut World2dField,
        _temperature: &mut World2dField,
        _humidity: &mut World2dField,
        _surface_water: &mut World2dField,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct World2dStats {
    /// (min, max, mean)
    pub altitude: (Scalar, Scalar, Scalar),
    /// (min, max, mean)
    pub temperature: (Scalar, Scalar, Scalar),
    /// (min, max, mean)
    pub humidity: (Scalar, Scalar, Scalar),
    /// (min, max, mean)
    pub surface_water: (Scalar, Scalar, Scalar),
}

pub struct World2d {
    size: usize,
    altitude: Switch<Grid2d<Scalar>>,
    temperature: Switch<Grid2d<Scalar>>,
    humidity: Switch<Grid2d<Scalar>>,
    surface_water: Switch<Grid2d<Scalar>>,
    simulation: Box<dyn World2dSimulation>,
    stats: World2dStats,
}

impl World2d {
    pub fn new(config: &World2dConfig, mut simulation: Box<dyn World2dSimulation>) -> Self {
        let mut altitude = {
            let gen = NoiseMapGenerator::new(config.altitude_seed, config.size, config.zoom);
            let diff = config.altitude_range.end - config.altitude_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.altitude_range.start + diff * v),
            )
        };
        let mut temperature = {
            let gen = NoiseMapGenerator::new(config.temperature_seed, config.size, config.zoom);
            let diff = config.temperature_range.end - config.temperature_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.temperature_range.start + diff * v),
            )
        };
        let mut humidity = {
            let gen = NoiseMapGenerator::new(config.humidity_seed, config.size, config.zoom);
            let diff = config.humidity_range.end - config.humidity_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.humidity_range.start + diff * v),
            )
        };
        let mut surface_water = Switch::new(2, Grid2d::new(config.size, config.size, 0.0));
        simulation.initialize_world(
            altitude.get_mut().unwrap(),
            temperature.get_mut().unwrap(),
            humidity.get_mut().unwrap(),
            surface_water.get_mut().unwrap(),
        );
        let mut result = Self {
            size: config.size,
            altitude,
            temperature,
            humidity,
            surface_water,
            simulation,
            stats: Default::default(),
        };
        result.calculate_stats();
        result
    }

    pub fn generate<FA, FT, FH, FSW>(
        size: usize,
        mut simulation: Box<dyn World2dSimulation>,
        mut altitude_generator: FA,
        mut temperature_generator: FT,
        mut humidity_generator: FH,
        mut surface_water_generator: FSW,
    ) -> Self
    where
        FA: FnMut(usize, usize) -> Scalar,
        FT: FnMut(usize, usize) -> Scalar,
        FH: FnMut(usize, usize) -> Scalar,
        FSW: FnMut(usize, usize) -> Scalar,
    {
        let altitude = (0..(size * size))
            .map(|i| altitude_generator(i % size, i / size))
            .collect::<Vec<_>>();
        let temperature = (0..size * size)
            .map(|i| temperature_generator(i % size, i / size))
            .collect::<Vec<_>>();
        let humidity = (0..size * size)
            .map(|i| humidity_generator(i % size, i / size))
            .collect::<Vec<_>>();
        let surface_water = (0..size * size)
            .map(|i| surface_water_generator(i % size, i / size))
            .collect::<Vec<_>>();
        let mut altitude = Switch::new(2, Grid2d::with_cells(size, altitude));
        let mut temperature = Switch::new(2, Grid2d::with_cells(size, temperature));
        let mut humidity = Switch::new(2, Grid2d::with_cells(size, humidity));
        let mut surface_water = Switch::new(2, Grid2d::with_cells(size, surface_water));
        simulation.initialize_world(
            altitude.get_mut().unwrap(),
            temperature.get_mut().unwrap(),
            humidity.get_mut().unwrap(),
            surface_water.get_mut().unwrap(),
        );
        let mut result = Self {
            size,
            altitude,
            temperature,
            humidity,
            surface_water,
            simulation,
            stats: Default::default(),
        };
        result.calculate_stats();
        result
    }

    pub fn stats(&self) -> &World2dStats {
        &self.stats
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn altitude(&self) -> &Grid2d<Scalar> {
        self.altitude.get().unwrap()
    }

    pub fn temperature(&self) -> &Grid2d<Scalar> {
        self.temperature.get().unwrap()
    }

    pub fn humidity(&self) -> &Grid2d<Scalar> {
        self.humidity.get().unwrap()
    }

    pub fn surface_water(&self) -> &Grid2d<Scalar> {
        self.surface_water.get().unwrap()
    }

    pub fn simulation(&self) -> &dyn World2dSimulation {
        self.simulation.borrow()
    }

    pub fn as_simulation<T>(&self) -> Option<&T>
    where
        T: World2dSimulation,
    {
        self.simulation.as_any().downcast_ref::<T>()
    }

    pub fn process(&mut self) {
        self.simulation.process_world(
            &mut self.altitude,
            &mut self.temperature,
            &mut self.humidity,
            &mut self.surface_water,
        );
        self.calculate_stats();
    }

    pub fn remap_region<F, T>(&self, mut range: Range<(usize, usize)>, mut f: F) -> Grid2d<T>
    where
        // (col, row, altitude, temperature, humidity, surface water)
        F: FnMut(usize, usize, Scalar, Scalar, Scalar, Scalar) -> T,
        T: Clone + Send + Sync,
    {
        range.end.0 = range.end.0.min(self.size);
        range.end.1 = range.end.1.min(self.size);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cells = self
            .altitude
            .get()
            .unwrap()
            .iter_view(range.clone())
            .zip(self.temperature.get().unwrap().iter_view(range.clone()))
            .zip(self.humidity.get().unwrap().iter_view(range.clone()))
            .zip(self.surface_water.get().unwrap().iter_view(range.clone()))
            .map(|((((col, row, a), (_, _, t)), (_, _, h)), (_, _, sw))| {
                f(col, row, *a, *t, *h, *sw)
            })
            .collect::<Vec<T>>();
        Grid2d::with_cells(range.end.0 - range.start.0, cells)
    }

    pub fn resample_region<F, T>(
        &self,
        mut range: Range<(usize, usize)>,
        margin: usize,
        mut f: F,
    ) -> Grid2d<T>
    where
        // (col, row, altitude, temperature, humidity, surface water)
        F: FnMut(
            usize,
            usize,
            Grid2d<&Scalar>,
            Grid2d<&Scalar>,
            Grid2d<&Scalar>,
            Grid2d<&Scalar>,
        ) -> T,
        T: Clone + Send + Sync,
    {
        range.end.0 = range.end.0.min(self.size);
        range.end.1 = range.end.1.min(self.size);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cells = self
            .altitude
            .get()
            .unwrap()
            .iter_sample(range.clone(), margin)
            .zip(
                self.temperature
                    .get()
                    .unwrap()
                    .iter_sample(range.clone(), margin),
            )
            .zip(
                self.humidity
                    .get()
                    .unwrap()
                    .iter_sample(range.clone(), margin),
            )
            .zip(
                self.surface_water
                    .get()
                    .unwrap()
                    .iter_sample(range.clone(), margin),
            )
            .map(|((((col, row, a), (_, _, t)), (_, _, h)), (_, _, sw))| f(col, row, a, t, h, sw))
            .collect::<Vec<T>>();
        Grid2d::with_cells(range.end.0 - range.start.0, cells)
    }

    fn calculate_stats(&mut self) {
        self.stats.altitude = {
            let (min, max, accum) = self
                .altitude
                .get()
                .unwrap()
                .iter()
                .fold((SCALAR_INFINITY, SCALAR_NEG_INFINITY, 0.0), |a, v| {
                    (a.0.min(*v), a.1.max(*v), a.2 + *v)
                });
            (
                min,
                max,
                accum / self.altitude.get().unwrap().len() as Scalar,
            )
        };
        self.stats.temperature = {
            let (min, max, accum) = self
                .temperature
                .get()
                .unwrap()
                .iter()
                .fold((SCALAR_INFINITY, SCALAR_NEG_INFINITY, 0.0), |a, v| {
                    (a.0.min(*v), a.1.max(*v), a.2 + *v)
                });
            (
                min,
                max,
                accum / self.altitude.get().unwrap().len() as Scalar,
            )
        };
        self.stats.humidity = {
            let (min, max, accum) = self
                .humidity
                .get()
                .unwrap()
                .iter()
                .fold((SCALAR_INFINITY, SCALAR_NEG_INFINITY, 0.0), |a, v| {
                    (a.0.min(*v), a.1.max(*v), a.2 + *v)
                });
            (
                min,
                max,
                accum / self.altitude.get().unwrap().len() as Scalar,
            )
        };
        self.stats.surface_water = {
            let (min, max, accum) = self
                .surface_water
                .get()
                .unwrap()
                .iter()
                .fold((SCALAR_INFINITY, SCALAR_NEG_INFINITY, 0.0), |a, v| {
                    (a.0.min(*v), a.1.max(*v), a.2 + *v)
                });
            (
                min,
                max,
                accum / self.altitude.get().unwrap().len() as Scalar,
            )
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World2dData<S>
where
    S: World2dSimulation,
{
    size: usize,
    altitude: Grid2d<Scalar>,
    temperature: Grid2d<Scalar>,
    humidity: Grid2d<Scalar>,
    surface_water: Grid2d<Scalar>,
    simulation: S,
}

impl<S> From<&World2d> for World2dData<S>
where
    S: World2dSimulation + Clone,
{
    fn from(world: &World2d) -> Self {
        Self {
            size: world.size,
            altitude: world.altitude.get().unwrap().clone(),
            temperature: world.temperature.get().unwrap().clone(),
            humidity: world.humidity.get().unwrap().clone(),
            surface_water: world.surface_water.get().unwrap().clone(),
            simulation: world.as_simulation::<S>().unwrap().clone(),
        }
    }
}

impl<S> From<&World2dData<S>> for World2d
where
    S: World2dSimulation + Clone,
{
    fn from(data: &World2dData<S>) -> Self {
        Self {
            size: data.size,
            altitude: Switch::new(2, data.altitude.clone()),
            temperature: Switch::new(2, data.temperature.clone()),
            humidity: Switch::new(2, data.humidity.clone()),
            surface_water: Switch::new(2, data.surface_water.clone()),
            simulation: Box::new(data.simulation.clone()),
            stats: Default::default(),
        }
    }
}
