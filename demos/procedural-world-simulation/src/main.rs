#![allow(clippy::absurd_extreme_comparisons)]

extern crate oxygengine_procedural as procedural;

mod data_aggregator;

use data_aggregator::*;
use minifb::{Key, KeyRepeat, MouseMode, Scale, Window, WindowOptions};
use procedural::prelude::*;
use std::f64::consts::PI;

const SIZE: usize = 100;
const ALTITUDE_LIMIT: Scalar = 200.0;
const HUMIDITY_LIMIT: Scalar = 0.25;
const TEMPERATURE_LIMIT: Scalar = 100.0;
const WATER_LIMIT: Scalar = 30.0;
const VELOCITY_LIMIT: Scalar = 1.0;
const DIVERGENCE_LIMIT: Scalar = 1.0;
const PRESSURE_LIMIT: Scalar = 1.0;
const SLOPENESS_LIMIT: Scalar = 0.5;
const STEPS_LIMIT: usize = 0;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum VisualisationMode {
    Altitude,
    Temperature,
    Humidity,
    SurfaceLevel,
    Biome,
    Velocity,
    Divergence,
    Pressure,
    Slopeness,
}

fn build_world(altitude_seed: u32) -> World2d {
    println!("BUILD WORLD");
    let simulation = {
        let config = World2dClimateSimulationConfig {
            full_year_steps: 364 * 5,
            water_capacity: WATER_LIMIT,
            altitude_range: 0.0..ALTITUDE_LIMIT,
            temperature_range: 0.0..TEMPERATURE_LIMIT,
            world_axis_angle: 0.0 * PI as Scalar / 180.0,
            mass_diffuse_factor: 0.00001,
            // mass_diffuse_factor: 1.0,
            // viscosity_factor: 0.00001,
            viscosity_factor: 1.0,
            viscosity_iterations: 10,
            //mass_diffuse_iterations: 10,
            poisson_pressure_iterations: 10,
            world_core_heating: 0.0,
            sun_heating: 0.0,
            thermal_radiation: 0.1,
            sun_heating_adaptive_correction_factor: 1.0,
            ..Default::default()
        };
        World2dClimateSimulation::new(config)
    };
    let config = World2dConfig {
        size: SIZE,
        zoom: 10.0,
        altitude_range: 0.0..(ALTITUDE_LIMIT * 0.5),
        temperature_range: 0.0..TEMPERATURE_LIMIT,
        altitude_seed,
        temperature_seed: rand::random(),
        humidity_seed: rand::random(),
        ..Default::default()
    };
    World2d::new(&config, Box::new(simulation))

    // World2d::generate(
    //     SIZE,
    //     Box::new(simulation),
    //     |_, _| 0.0,
    //     // |col, row| {
    //     //     let dx = 74.0 - col as f64;
    //     //     let dy = 44.0 - row as f64;
    //     //     let f = ((dx * dx + dy * dy) / (10.0 * 10.0)).max(0.0).min(1.0);
    //     //     ALTITUDE_LIMIT * (0.9 + 0.1 * (1.0 - f * f * f * f * f * f))
    //     // },
    //     // |col, row| {
    //     //     let f = 1.0 - (0.5 + col as f64 / SIZE as f64 - row as f64 / SIZE as f64).max(0.0).min(1.0);
    //     //     ALTITUDE_LIMIT * f
    //     // },
    //     // |col, row| if col - 25 > row { 0.0 } else { ALTITUDE_LIMIT },
    //     // |col, row| if col - 25 > row { ALTITUDE_LIMIT } else { 0.0 },
    //     // |col, row| ALTITUDE_LIMIT * col as f64 / SIZE as f64,
    //     // |col, _| if col < SIZE - 20 { 0.0 } else { ALTITUDE_LIMIT },
    //     |col, row| if col % 2 == 0 { TEMPERATURE_LIMIT * row as f64 / SIZE as f64 } else { 0.0 },
    //     // |_, _| rand::random::<f64>() * TEMPERATURE_LIMIT,
    //     |_, _| 0.0,
    //     |_, _| 0.0,
    // )
}

fn main() {
    let mut steps = 0;
    let mut auto = false;
    let mut mode = VisualisationMode::Temperature;
    let mut altitude_seed = if let Some(seed) = ::std::env::args().skip(1).last() {
        if let Ok(seed) = seed.parse() {
            seed
        } else {
            rand::random()
        }
    } else {
        rand::random()
    };
    let mut data_aggregator = DataAggregator::new("resources/data.txt");
    data_aggregator.set_auto_flush(Some(100));

    println!("SEED: {}", altitude_seed);
    println!("CREATE WINDOW");
    let options = WindowOptions {
        scale: Scale::X8,
        resize: false,
        ..Default::default()
    };
    let mut window = Window::new(
        &format!("Procedural World Simulation - {:?}", mode),
        SIZE,
        SIZE,
        options,
    )
    .unwrap();

    let mut world = build_world(altitude_seed);
    let buffer = world_to_buffer(mode, &world);
    window.update_with_buffer(&buffer, SIZE, SIZE).unwrap();

    println!("LOOP START");
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut dirty = false;
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            altitude_seed = rand::random();
            println!("SEED: {}", altitude_seed);
            world = build_world(altitude_seed);
            dirty = true;
        } else if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            mode = VisualisationMode::Altitude;
            dirty = true;
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            mode = VisualisationMode::Temperature;
            dirty = true;
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            mode = VisualisationMode::Humidity;
            dirty = true;
        } else if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            mode = VisualisationMode::SurfaceLevel;
            dirty = true;
        } else if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            mode = VisualisationMode::Biome;
            dirty = true;
        } else if window.is_key_pressed(Key::Key6, KeyRepeat::No) {
            mode = VisualisationMode::Velocity;
            dirty = true;
        } else if window.is_key_pressed(Key::Key7, KeyRepeat::No) {
            mode = VisualisationMode::Divergence;
            dirty = true;
        } else if window.is_key_pressed(Key::Key8, KeyRepeat::No) {
            mode = VisualisationMode::Pressure;
            dirty = true;
        } else if window.is_key_pressed(Key::Key9, KeyRepeat::No) {
            mode = VisualisationMode::Slopeness;
            dirty = true;
        } else if window.is_key_pressed(Key::P, KeyRepeat::No) {
            auto = !auto;
            dirty = true;
        }
        if auto
            || window.is_key_pressed(Key::Space, KeyRepeat::No)
            || window.is_key_down(Key::Enter)
        {
            let timer = ::std::time::Instant::now();
            world.process();
            println!("PROCESSED IN: {:?}", timer.elapsed());
            dirty = true;
            let sun_heating = &world
                .as_simulation::<World2dClimateSimulation>()
                .unwrap()
                .config()
                .sun_heating;
            data_aggregator.push(*sun_heating);
            steps += 1;
            if STEPS_LIMIT > 0 && steps >= STEPS_LIMIT {
                break;
            }
        }
        if window.is_key_pressed(Key::I, KeyRepeat::No) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                show_cell_info(&world, x as usize % SIZE, y as usize % SIZE);
            }
        }
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            println!("WORLD STATS: {:#?}", world.stats());
            let sim = world.as_simulation::<World2dClimateSimulation>().unwrap();
            println!("SUN HEATING: {}", sim.config().sun_heating);
            println!("STEP: {}", steps);
        }
        if dirty {
            let (year, day) = {
                let sim = world.as_simulation::<World2dClimateSimulation>().unwrap();
                (sim.years(), sim.steps())
            };
            window.set_title(&format!(
                "Procedural World Simulation - {:?} | Year: {} | Day: {}",
                mode, year, day
            ));
            window
                .update_with_buffer(&world_to_buffer(mode, &world), SIZE, SIZE)
                .unwrap();
        } else {
            window.update();
        }
    }
    println!("LOOP END");
    println!("SEED: {}", altitude_seed);
}

fn show_cell_info(world: &World2d, x: usize, y: usize) {
    let altitude = world.altitude()[(x, y)];
    let temperature = world.temperature()[(x, y)];
    let humidity = world.humidity()[(x, y)];
    let surface_water = world.surface_water()[(x, y)];
    let velocity = if let Some(velocity) = world
        .as_simulation::<World2dClimateSimulation>()
        .unwrap()
        .velocity()
    {
        velocity[(x, y)].into()
    } else {
        (0.0, 0.0)
    };
    println!(
        "CELL INFO {} x {}\n- altitude: {}\n- temperature: {}\n- humidity: {}\n- surface water: {}\n- velocity: {:?}",
        x, y, altitude, temperature, humidity, surface_water, velocity
    );
}

fn world_to_buffer(mode: VisualisationMode, world: &World2d) -> Vec<u32> {
    match mode {
        VisualisationMode::Altitude => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, altitude, _, _, _| {
                let v = (255.0 * altitude / ALTITUDE_LIMIT).max(0.0).min(255.0) as u8;
                let v = u32::from(v);
                v | v << 8 | v << 16
            })
            .into(),
        VisualisationMode::Temperature => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, _, temperature, _, _| {
                let f = (temperature / TEMPERATURE_LIMIT).max(0.0).min(1.0);
                if f >= 0.5 {
                    let f = (f - 0.5) * 2.0;
                    let rv = (255.0 * f).max(0.0).min(255.0) as u8;
                    let rv = u32::from(rv);
                    let gv = (255.0 * (1.0 - f)).max(0.0).min(255.0) as u8;
                    let gv = u32::from(gv);
                    rv << 16 | gv << 8
                } else {
                    let f = f * 2.0;
                    let gv = (255.0 * f).max(0.0).min(255.0) as u8;
                    let gv = u32::from(gv);
                    let bv = (255.0 * (1.0 - f)).max(0.0).min(255.0) as u8;
                    let bv = u32::from(bv);
                    gv << 8 | bv
                }
            })
            .into(),
        VisualisationMode::Humidity => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, _, _, humidity, _| {
                let v = (255.0 * humidity / HUMIDITY_LIMIT).max(0.0).min(255.0) as u8;
                let v = u32::from(v);
                v | v << 8 | v << 16
            })
            .into(),
        VisualisationMode::SurfaceLevel => world
            .remap_region(
                (0, 0)..(SIZE, SIZE),
                |_, _, altitude, _, _, surface_water| {
                    let v = (255.0 * (altitude + surface_water) / ALTITUDE_LIMIT)
                        .max(0.0)
                        .min(255.0) as u8;
                    let v = u32::from(v);
                    v | v << 8 | v << 16
                },
            )
            .into(),
        VisualisationMode::Biome => world
            .remap_region(
                (0, 0)..(SIZE, SIZE),
                |_, _, altitude, temperature, _, surface_water| {
                    let t = if surface_water < 10.0 {
                        if temperature < 40.0 {
                            0
                        } else if temperature < 90.0 {
                            1
                        } else {
                            2
                        }
                    } else if temperature < 15.0 {
                        0
                    } else {
                        1
                    };
                    if t == 0 {
                        let g = (128.0 + 127.0 * altitude / ALTITUDE_LIMIT)
                            .max(0.0)
                            .min(255.0) as u8;
                        let g = u32::from(g);
                        g | g << 8 | g << 16
                    } else if t == 1 {
                        let g = (55.0 + 200.0 * altitude / ALTITUDE_LIMIT)
                            .max(0.0)
                            .min(255.0) as u8;
                        let g = u32::from(g);
                        let swf = 1.0 - surface_water / WATER_LIMIT;
                        let swf = 1.0 - swf * swf * swf;
                        let w = (192.0 * swf).max(0.0).min(255.0) as u8;
                        let w = u32::from(w);
                        w | g << 8
                    } else {
                        let g = (92.0 + 127.0 * altitude / ALTITUDE_LIMIT)
                            .max(0.0)
                            .min(255.0) as u8;
                        let g = u32::from(g);
                        0x30 | g << 8 | g << 16
                    }
                },
            )
            .into(),
        VisualisationMode::Velocity => {
            if let Some(velocity) = world
                .as_simulation::<World2dClimateSimulation>()
                .unwrap()
                .velocity()
            {
                velocity
                    .cells()
                    .iter()
                    .map(|vel| {
                        let x = (((vel.0 / VELOCITY_LIMIT) + 1.0) * 0.5 * 255.0)
                            .max(0.0)
                            .min(255.0) as u8;
                        let x = u32::from(x);
                        let y = (((vel.1 / VELOCITY_LIMIT) + 1.0) * 0.5 * 255.0)
                            .max(0.0)
                            .min(255.0) as u8;
                        let y = u32::from(y);
                        y | x << 16
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![0x0088_0088; SIZE * SIZE]
            }
        }
        VisualisationMode::Divergence => {
            if let Some(divergence) = world
                .as_simulation::<World2dClimateSimulation>()
                .unwrap()
                .divergence()
            {
                divergence
                    .cells()
                    .iter()
                    .map(|div| {
                        let p = (div / DIVERGENCE_LIMIT).max(0.0);
                        let n = -(div / DIVERGENCE_LIMIT).min(0.0);
                        let vp = (p * 255.0).max(0.0).min(255.0) as u8;
                        let vp = u32::from(vp);
                        let vn = (n * 255.0).max(0.0).min(255.0) as u8;
                        let vn = u32::from(vn);
                        vn | vp << 16
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![0x0000_0000; SIZE * SIZE]
            }
        }
        VisualisationMode::Pressure => {
            if let Some(pressure) = world
                .as_simulation::<World2dClimateSimulation>()
                .unwrap()
                .pressure()
            {
                pressure
                    .cells()
                    .iter()
                    .map(|pres| {
                        let p = (pres / PRESSURE_LIMIT).max(0.0);
                        let n = -(pres / PRESSURE_LIMIT).min(0.0);
                        let vp = (p * 255.0).max(0.0).min(255.0) as u8;
                        let vp = u32::from(vp);
                        let vn = (n * 255.0).max(0.0).min(255.0) as u8;
                        let vn = u32::from(vn);
                        vn | vp << 16
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![0x0000_0000; SIZE * SIZE]
            }
        }
        VisualisationMode::Slopeness => {
            if let Some(slopeness) = world
                .as_simulation::<World2dClimateSimulation>()
                .unwrap()
                .slopeness()
            {
                slopeness
                    .cells()
                    .iter()
                    .map(|vel| {
                        let x = ((vel.0 / SLOPENESS_LIMIT + 1.0) * 0.5 * 255.0)
                            .max(0.0)
                            .min(255.0) as u8;
                        let x = u32::from(x);
                        let y = ((vel.1 / SLOPENESS_LIMIT + 1.0) * 0.5 * 255.0)
                            .max(0.0)
                            .min(255.0) as u8;
                        let y = u32::from(y);
                        y | x << 16
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![0x0088_0088; SIZE * SIZE]
            }
        }
    }
}
