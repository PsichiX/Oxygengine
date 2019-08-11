use crate::components::PlayerTag;
use amethyst::{
    core::{ecs::Component, transform::Transform},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::rgb::Srgba, Camera},
    window::ScreenDimensions,
};
use nav::prelude::*;

#[derive(Default)]
pub struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.add_resource(NavMeshesRes::default());
        let dimensions = world.read_resource::<ScreenDimensions>().clone();
        init_camera(world, &dimensions);
        init_nav_mesh(world);
        init_agent::<PlayerTag>(world, 400.0, 450.0, 100.0);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }

        Trans::None
    }
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

fn init_nav_mesh(world: &mut World) {
    let vertices: Vec<NavVec3> = vec![
        (50.0, 50.0).into(),   // 0
        (500.0, 50.0).into(),  // 1
        (500.0, 100.0).into(), // 2
        (100.0, 100.0).into(), // 3
        (100.0, 300.0).into(), // 4
        (700.0, 300.0).into(), // 5
        (700.0, 50.0).into(),  // 6
        (750.0, 50.0).into(),  // 7
        (750.0, 550.0).into(), // 8
        (50.0, 550.0).into(),  // 9
    ];
    let triangles: Vec<NavTriangle> = vec![
        (1, 2, 3).into(), // 0
        (0, 1, 3).into(), // 1
        (0, 3, 4).into(), // 2
        (0, 4, 9).into(), // 3
        (4, 8, 9).into(), // 4
        (4, 5, 8).into(), // 5
        (5, 7, 8).into(), // 6
        (5, 6, 7).into(), // 7
    ];

    let mut debug = DebugLinesComponent::default();
    for triangle in &triangles {
        let f = vertices[triangle.first as usize];
        let s = vertices[triangle.second as usize];
        let t = vertices[triangle.third as usize];
        debug.add_line(
            [f.x as f32, f.y as f32, f.z as f32].into(),
            [s.x as f32, s.y as f32, s.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
        debug.add_line(
            [s.x as f32, s.y as f32, s.z as f32].into(),
            [t.x as f32, t.y as f32, t.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
        debug.add_line(
            [t.x as f32, t.y as f32, t.z as f32].into(),
            [f.x as f32, f.y as f32, f.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
    }
    world
        .create_entity()
        .with(Transform::default())
        .with(debug)
        .build();

    let mesh = NavMesh::new(vertices, triangles).unwrap();
    world.write_resource::<NavMeshesRes>().register(mesh);
}

fn init_agent<T>(world: &mut World, x: f32, y: f32, speed: f64)
where
    T: Component + Default + Copy + Send + Sync,
{
    let mut agent = NavAgent::new((x as f64, y as f64).into());
    agent.speed = speed;
    world
        .create_entity()
        .with(agent)
        .with(SimpleNavDriverTag)
        .with(DebugLinesComponent::default())
        .with(T::default())
        .build();
}
