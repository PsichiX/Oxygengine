use oxygengine_utils::{grid_2d::Grid2d, noise_map_generator::NoiseMapGenerator};
use psyche_utils::switch::Switch;
use std::{any::Any, borrow::Borrow, ops::Range};

pub type World2dField = Switch<Grid2d<f64>>;

#[derive(Debug, Clone)]
pub struct World2dConfig {
    pub size: usize,
    pub zoom: f64,
    pub altitude_seed: u32,
    pub altitude_range: Range<f64>,
    pub temperature_seed: u32,
    pub temperature_range: Range<f64>,
    pub humidity_seed: u32,
    pub humidity_range: Range<f64>,
}

impl Default for World2dConfig {
    fn default() -> Self {
        Self {
            size: 160,
            zoom: 5.0,
            altitude_seed: 1,
            altitude_range: 0.0..100.0,
            temperature_seed: 2,
            temperature_range: -50.0..50.0,
            humidity_seed: 3,
            humidity_range: 0.1..1.0,
        }
    }
}

pub trait World2dSimulation: Any + Send + Sync {
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
    pub altitude: (f64, f64, f64),
    /// (min, max, mean)
    pub temperature: (f64, f64, f64),
    /// (min, max, mean)
    pub humidity: (f64, f64, f64),
    /// (min, max, mean)
    pub surface_water: (f64, f64, f64),
}

pub struct World2d {
    size: usize,
    altitude: Switch<Grid2d<f64>>,
    temperature: Switch<Grid2d<f64>>,
    humidity: Switch<Grid2d<f64>>,
    surface_water: Switch<Grid2d<f64>>,
    simulation: Box<dyn World2dSimulation>,
    stats: World2dStats,
}

impl World2d {
    pub fn new(config: &World2dConfig, simulation: Box<dyn World2dSimulation>) -> Self {
        let altitude = {
            let gen = NoiseMapGenerator::new(config.altitude_seed, config.size, config.zoom);
            let diff = config.altitude_range.end - config.altitude_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.altitude_range.start + diff * v),
            )
        };
        let temperature = {
            let gen = NoiseMapGenerator::new(config.temperature_seed, config.size, config.zoom);
            let diff = config.temperature_range.end - config.temperature_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.temperature_range.start + diff * v),
            )
        };
        let humidity = {
            let gen = NoiseMapGenerator::new(config.humidity_seed, config.size, config.zoom);
            let diff = config.humidity_range.end - config.humidity_range.start;
            Switch::new(
                2,
                gen.build_chunk((0, 0), 0)
                    .map(|_, _, v| config.humidity_range.start + diff * v),
            )
        };
        let surface_water = Switch::new(2, Grid2d::new(config.size, config.size, 0.0));
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

    pub fn generate<F>(
        size: usize,
        simulation: Box<dyn World2dSimulation>,
        mut altitude_generator: F,
        mut temperature_generator: F,
        mut humidity_generator: F,
        mut surface_water_generator: F,
    ) -> Self
    where
        F: FnMut(usize, usize) -> f64,
    {
        let altitude = (0..(size * size))
            .map(|i| altitude_generator(i % size, i / size))
            .collect::<Vec<f64>>();
        let temperature = (0..size * size)
            .map(|i| temperature_generator(i % size, i / size))
            .collect::<Vec<f64>>();
        let humidity = (0..size * size)
            .map(|i| humidity_generator(i % size, i / size))
            .collect::<Vec<f64>>();
        let surface_water = (0..size * size)
            .map(|i| surface_water_generator(i % size, i / size))
            .collect::<Vec<f64>>();
        let altitude = Switch::new(2, Grid2d::with_cells(size, altitude));
        let temperature = Switch::new(2, Grid2d::with_cells(size, temperature));
        let humidity = Switch::new(2, Grid2d::with_cells(size, humidity));
        let surface_water = Switch::new(2, Grid2d::with_cells(size, surface_water));
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

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn altitude(&self) -> &Grid2d<f64> {
        self.altitude.get().unwrap()
    }

    pub fn temperature(&self) -> &Grid2d<f64> {
        self.temperature.get().unwrap()
    }

    pub fn humidity(&self) -> &Grid2d<f64> {
        self.humidity.get().unwrap()
    }

    pub fn surface_water(&self) -> &Grid2d<f64> {
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
        F: FnMut(usize, usize, f64, f64, f64, f64) -> T,
        T: Clone,
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
        F: FnMut(usize, usize, Grid2d<&f64>, Grid2d<&f64>, Grid2d<&f64>, Grid2d<&f64>) -> T,
        T: Clone,
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
            let (min, max) = self
                .altitude
                .get()
                .unwrap()
                .iter()
                .fold((std::f64::INFINITY, std::f64::NEG_INFINITY), |a, v| {
                    (a.0.min(*v), a.1.max(*v))
                });
            (min, max, (min + max) * 0.5)
        };
        self.stats.temperature = {
            let (min, max) = self
                .temperature
                .get()
                .unwrap()
                .iter()
                .fold((std::f64::INFINITY, std::f64::NEG_INFINITY), |a, v| {
                    (a.0.min(*v), a.1.max(*v))
                });
            (min, max, (min + max) * 0.5)
        };
        self.stats.humidity = {
            let (min, max) = self
                .humidity
                .get()
                .unwrap()
                .iter()
                .fold((std::f64::INFINITY, std::f64::NEG_INFINITY), |a, v| {
                    (a.0.min(*v), a.1.max(*v))
                });
            (min, max, (min + max) * 0.5)
        };
        self.stats.surface_water = {
            let (min, max) = self
                .surface_water
                .get()
                .unwrap()
                .iter()
                .fold((std::f64::INFINITY, std::f64::NEG_INFINITY), |a, v| {
                    (a.0.min(*v), a.1.max(*v))
                });
            (min, max, (min + max) * 0.5)
        };
    }
}
