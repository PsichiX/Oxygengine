extern crate oxygengine_procedural as procedural;

use minifb::{Key, KeyRepeat, MouseMode, Scale, Window, WindowOptions};
use procedural::prelude::*;

const SIZE: usize = 60;
const ALTITUDE_LIMIT: f64 = 200.0;
const TEMPERATURE_LIMIT: f64 = 100.0;
const WATER_LIMIT: f64 = 30.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum VisualisationMode {
    Altitude,
    Temperature,
    Humidity,
    SurfaceLevel,
    Biome,
    Landmass,
    Combined,
}

fn build_world(altitude_seed: u32) -> World2d {
    println!("BUILD WORLD");
    let simulation = {
        let mut config = World2dClimateSimulationConfig::default();
        config.water_capacity = WATER_LIMIT;
        config.temperature_range = 0.0..TEMPERATURE_LIMIT;
        config.full_year_steps = 100;
        config.world_axis_angle = 10.0 * ::std::f64::consts::PI / 180.0;
        World2dClimateSimulation::new(config)
    };
    let mut config = World2dConfig::default();
    config.size = SIZE;
    config.zoom = 10.0;
    config.altitude_range = 0.0..(ALTITUDE_LIMIT * 0.5);
    config.temperature_range = 0.0..TEMPERATURE_LIMIT;
    config.altitude_seed = altitude_seed;
    config.temperature_seed = rand::random();
    config.humidity_seed = rand::random();
    World2d::new(&config, Box::new(simulation))
}

fn main() {
    let mut mode = VisualisationMode::Combined;
    let mut altitude_seed = if let Some(seed) = ::std::env::args().skip(1).last() {
        if let Ok(seed) = seed.parse() {
            seed
        } else {
            rand::random()
        }
    } else {
        rand::random()
    };

    println!("SEED: {}", altitude_seed);
    println!("CREATE WINDOW");
    let mut options = WindowOptions::default();
    options.scale = Scale::X8;
    options.resize = false;
    let mut window = Window::new(
        &format!("Procedural World Simulation - {:?}", mode),
        if mode == VisualisationMode::Combined {
            SIZE * 3
        } else {
            SIZE
        },
        SIZE,
        options,
    )
    .unwrap();

    let mut world = build_world(altitude_seed);
    let buffer = world_to_buffer(mode, &world);
    window.update_with_buffer(&buffer).unwrap();

    println!("LOOP START");
    let mut last_combined = mode == VisualisationMode::Combined;
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
            mode = VisualisationMode::Landmass;
            dirty = true;
        } else if window.is_key_pressed(Key::Key7, KeyRepeat::No) {
            mode = VisualisationMode::Combined;
            dirty = true;
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) || window.is_key_down(Key::Enter) {
            world.process();
            dirty = true;
        }
        if window.is_key_pressed(Key::I, KeyRepeat::No) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                show_cell_info(&world, x as usize, y as usize);
            }
        }
        if dirty {
            if (mode == VisualisationMode::Combined) != last_combined {
                last_combined = mode == VisualisationMode::Combined;
                println!("CREATE WINDOW");
                let mut options = WindowOptions::default();
                options.scale = Scale::X8;
                options.resize = false;
                window = Window::new(
                    &format!("Procedural World Simulation - {:?}", mode),
                    if mode == VisualisationMode::Combined {
                        SIZE * 3
                    } else {
                        SIZE
                    },
                    SIZE,
                    options,
                )
                .unwrap();
            }
            let (year, day) = {
                let sim = world.as_simulation::<World2dClimateSimulation>().unwrap();
                (sim.years(), sim.steps())
            };
            window.set_title(&format!(
                "Procedural World Simulation - {:?} | Year: {} | Day: {}",
                mode, year, day
            ));
            window
                .update_with_buffer(&world_to_buffer(mode, &world))
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
    println!(
        "CELL INFO {} x {}\n- altitude: {}\n- temperature: {}\n- humidity: {}\n- surface water: {}",
        x, y, altitude, temperature, humidity, surface_water
    );
}

fn world_to_buffer(mode: VisualisationMode, world: &World2d) -> Vec<u32> {
    match mode {
        VisualisationMode::Altitude => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, altitude, _, _, _| {
                let v = (255.0 * altitude / ALTITUDE_LIMIT).max(0.0).min(255.0) as u8;
                let v = v as u32;
                v | v << 8 | v << 16
            })
            .into(),
        VisualisationMode::Temperature => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, _, temperature, _, _| {
                let f = (temperature / TEMPERATURE_LIMIT).max(0.0).min(1.0);
                if f >= 0.5 {
                    let f = (f - 0.5) * 2.0;
                    let rv = (255.0 * f).max(0.0).min(255.0) as u8;
                    let rv = rv as u32;
                    let gv = (255.0 * (1.0 - f)).max(0.0).min(255.0) as u8;
                    let gv = gv as u32;
                    rv << 16 | gv << 8
                } else {
                    let f = f * 2.0;
                    let gv = (255.0 * f).max(0.0).min(255.0) as u8;
                    let gv = gv as u32;
                    let bv = (255.0 * (1.0 - f)).max(0.0).min(255.0) as u8;
                    let bv = bv as u32;
                    gv << 8 | bv
                }
            })
            .into(),
        VisualisationMode::Humidity => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, _, _, humidity, _| {
                let v = (255.0 * humidity).max(0.0).min(255.0) as u8;
                let v = v as u32;
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
                    let v = v as u32;
                    v | v << 8 | v << 16
                },
            )
            .into(),
        VisualisationMode::Biome => world
            .remap_region(
                (0, 0)..(SIZE, SIZE),
                |_, _, altitude, temperature, _, surface_water| {
                    let s = if surface_water < 10.0 {
                        temperature < 55.0
                    } else {
                        temperature < 30.0
                    };
                    if s {
                        let g = (128.0 + 127.0 * altitude / ALTITUDE_LIMIT)
                            .max(0.0)
                            .min(255.0) as u8;
                        let g = g as u32;
                        g | g << 8 | g << 16
                    } else {
                        let g = (55.0 + 200.0 * altitude / ALTITUDE_LIMIT)
                            .max(0.0)
                            .min(255.0) as u8;
                        let g = g as u32;
                        let swf = 1.0 - surface_water / WATER_LIMIT;
                        let swf = 1.0 - swf * swf * swf;
                        let w = (192.0 * swf).max(0.0).min(255.0) as u8;
                        let w = w as u32;
                        w | g << 8
                    }
                },
            )
            .into(),
        VisualisationMode::Landmass => world
            .remap_region((0, 0)..(SIZE, SIZE), |_, _, _, _, _, surface_water| {
                if surface_water < 10.0 {
                    0x00FFFFFF
                } else {
                    0
                }
            })
            .into(),
        VisualisationMode::Combined => {
            let temperature = world_to_buffer(VisualisationMode::Temperature, &world);
            let biome = world_to_buffer(VisualisationMode::Biome, &world);
            let surface = world_to_buffer(VisualisationMode::SurfaceLevel, &world);
            let mut buffer = vec![0; SIZE * 3 * SIZE];
            for row in 0..SIZE {
                let from = row * SIZE * 3;
                let to = from + SIZE;
                let dst = &mut buffer[from..to];
                let from = row * SIZE;
                let to = from + SIZE;
                let src = &temperature[from..to];
                dst.copy_from_slice(src);
                let from = row * SIZE * 3 + SIZE;
                let to = from + SIZE;
                let dst = &mut buffer[from..to];
                let from = row * SIZE;
                let to = from + SIZE;
                let src = &biome[from..to];
                dst.copy_from_slice(src);
                let from = row * SIZE * 3 + SIZE * 2;
                let to = from + SIZE;
                let dst = &mut buffer[from..to];
                let from = row * SIZE;
                let to = from + SIZE;
                let src = &surface[from..to];
                dst.copy_from_slice(src);
            }
            buffer
        }
    }
}
