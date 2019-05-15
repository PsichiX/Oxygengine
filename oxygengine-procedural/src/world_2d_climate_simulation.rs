use crate::world_2d::{World2dFieldPair, World2dSimulation};
use utils::grid_2d::Grid2d;

#[derive(Debug, Clone)]
pub struct World2dClimateSimulationConfig {
    pub cell_humidity_limit: f64,
    pub cell_water_capacity: f64,
    pub cell_rainfall_per_step_limit: f64,
}

impl Default for World2dClimateSimulationConfig {
    fn default() -> Self {
        Self {
            cell_humidity_limit: 0.3,
            cell_water_capacity: 50.0,
            cell_rainfall_per_step_limit: 0.01,
        }
    }
}

#[derive(Default)]
pub struct World2dClimateSimulation {
    config: World2dClimateSimulationConfig,
}

impl World2dClimateSimulation {
    pub fn new(config: World2dClimateSimulationConfig) -> Self {
        Self { config }
    }
}

impl World2dSimulation for World2dClimateSimulation {
    fn initialize_world(
        &mut self,
        altitude: &mut Grid2d<f64>,
        temperature: &mut Grid2d<f64>,
        humidity: &mut Grid2d<f64>,
        surface_water: &mut Grid2d<f64>,
    ) {
        for i in 0..surface_water.len() {
            let h = (humidity.cells()[i] - self.config.cell_humidity_limit);
            surface_water.cells_mut()[i] = h.max(0.0) * self.config.cell_water_capacity;
        }
    }

    fn process_world(
        &mut self,
        altitude: World2dFieldPair,
        temperature: World2dFieldPair,
        humidity: World2dFieldPair,
        surface_water: World2dFieldPair,
    ) {

    }
}
