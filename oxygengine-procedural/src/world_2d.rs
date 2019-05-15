use psyche_utils::switch::Switch;
use std::ops::Range;
use utils::{grid_2d::Grid2d, noise_map_generator::NoiseMapGenerator};

#[derive(Debug, Clone)]
pub struct World2dConfig {
    pub chunk_size: usize,
    pub zoom: f64,
    pub segments: usize,
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
            chunk_size: 16,
            zoom: 5.0,
            segments: 10,
            altitude_seed: 1,
            altitude_range: -50.0..50.0,
            temperature_seed: 10,
            temperature_range: -50.0..50.0,
            humidity_seed: 100,
            humidity_range: 0.1..1.0,
        }
    }
}

pub struct World2dFieldPair<'a> {
    pub prev: &'a Grid2d<f64>,
    pub next: &'a mut Grid2d<f64>,
}

impl<'a> From<(&'a Grid2d<f64>, &'a mut Grid2d<f64>)> for World2dFieldPair<'a> {
    fn from((prev, next): (&'a Grid2d<f64>, &'a mut Grid2d<f64>)) -> Self {
        Self { prev, next }
    }
}

pub trait World2dSimulation {
    fn initialize_world(
        &mut self,
        altitude: &mut Grid2d<f64>,
        temperature: &mut Grid2d<f64>,
        humidity: &mut Grid2d<f64>,
        surface_water: &mut Grid2d<f64>,
    ) {
    }

    fn process_world(
        &mut self,
        altitude: World2dFieldPair,
        temperature: World2dFieldPair,
        humidity: World2dFieldPair,
        surface_water: World2dFieldPair,
    );
}

impl World2dSimulation for () {
    fn process_world(
        &mut self,
        _altitude: World2dFieldPair,
        _temperature: World2dFieldPair,
        _humidity: World2dFieldPair,
        _surface_water: World2dFieldPair,
    ) {
    }
}

pub struct World2d<T>
where
    T: Clone,
{
    altitude: Switch<Grid2d<f64>>,
    temperature: Switch<Grid2d<f64>>,
    humidity: Switch<Grid2d<f64>>,
    surface_water: Switch<Grid2d<f64>>,
    tiles: Grid2d<T>,
    simulation: Box<dyn World2dSimulation>,
}

impl<T> World2d<T>
where
    T: Clone + Default,
{
    pub fn new<S>(config: World2dConfig, mut simulation: S) -> Self
    where
        S: World2dSimulation + 'static,
    {
        let mut altitude = {
            let gen = NoiseMapGenerator::new(config.altitude_seed, config.chunk_size, config.zoom);
            let diff = config.altitude_range.end - config.altitude_range.start;
            Switch::new(
                2,
                gen.build_chunks((0, 0)..(config.segments as isize, config.segments as isize))
                    .map(|_, _, v| config.altitude_range.start + diff * v),
            )
        };
        let mut temperature = {
            let gen =
                NoiseMapGenerator::new(config.temperature_seed, config.chunk_size, config.zoom);
            let diff = config.temperature_range.end - config.temperature_range.start;
            Switch::new(
                2,
                gen.build_chunks((0, 0)..(config.segments as isize, config.segments as isize))
                    .map(|_, _, v| config.temperature_range.start + diff * v),
            )
        };
        let mut humidity = {
            let gen = NoiseMapGenerator::new(config.humidity_seed, config.chunk_size, config.zoom);
            let diff = config.humidity_range.end - config.humidity_range.start;
            Switch::new(
                2,
                gen.build_chunks((0, 0)..(config.segments as isize, config.segments as isize))
                    .map(|_, _, v| config.humidity_range.start + diff * v),
            )
        };
        let mut surface_water = Switch::new(
            2,
            Grid2d::new(
                config.chunk_size * config.segments,
                config.chunk_size * config.segments,
                0.0,
            ),
        );
        let tiles = Grid2d::new(
            config.chunk_size * config.segments,
            config.chunk_size * config.segments,
            Default::default(),
        );
        simulation.initialize_world(
            altitude.get_mut().unwrap(),
            temperature.get_mut().unwrap(),
            humidity.get_mut().unwrap(),
            surface_water.get_mut().unwrap(),
        );
        Self {
            altitude,
            temperature,
            humidity,
            surface_water,
            tiles,
            simulation: Box::new(simulation),
        }
    }

    pub fn process(&mut self) {
        self.simulation.process_world(
            self.altitude.iterate().unwrap().into(),
            self.temperature.iterate().unwrap().into(),
            self.humidity.iterate().unwrap().into(),
            self.surface_water.iterate().unwrap().into(),
        );
    }
}
