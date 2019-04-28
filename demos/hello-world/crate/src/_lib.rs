extern crate oxygengine;

use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// component that tags entity as moved with keyboard.
#[derive(Debug, Default, Copy, Clone)]
pub struct KeyboardMovementTag;

impl Component for KeyboardMovementTag {
    // tag components are empty so they use `NullStorage`.
    type Storage = NullStorage<Self>;
}

// component that tells the speed of entity.
#[derive(Debug, Default, Copy, Clone)]
pub struct Speed(pub Scalar);

impl Component for Speed {
    // not all entities has speed so we use `VecStorage`.
    type Storage = VecStorage<Self>;
}

// system that moves tagged entities.
pub struct KeyboardMovementSystem;

impl<'s> System<'s> for KeyboardMovementSystem {
    type SystemData = (
        // we will read input.
        Read<'s, InputController>,
        // we will read delta time from app lifecycle.
        ReadExpect<'s, AppLifeCycle>,
        // we will read speed components.
        ReadStorage<'s, Speed>,
        // we will filter by tag.
        ReadStorage<'s, KeyboardMovementTag>,
        // we will write to transforms.
        WriteStorage<'s, CompositeTransform>,
    );

    fn run(
        &mut self,
        (input, lifecycle, speed, keyboard_movement, mut transforms): Self::SystemData,
    ) {
        let dt = lifecycle.delta_time_seconds() as Scalar;
        let hor = -input.axis_or_default("move-left") + input.axis_or_default("move-right");
        let ver = -input.axis_or_default("move-up") + input.axis_or_default("move-down");
        let offset = Vec2::new(hor, ver);

        for (_, speed, transform) in (&keyboard_movement, &speed, &mut transforms).join() {
            transform.set_translation(transform.get_translation() + offset * speed.0 * dt);
        }
    }
}

pub struct GameState;

impl State for GameState {
    fn on_enter(&mut self, world: &mut World) {
        // create entity with camera to view scene.
        world
            .create_entity()
            .with(CompositeCamera::new(CompositeScalingMode::CenterAspect))
            .with(CompositeTransform::scale(400.0.into()))
            .build();

        // create player entity.
        let player = world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::red(),
                    rect: [-50.0, -50.0, 100.0, 100.0].into(),
                }
                .into(),
            ))
            .with(CompositeTransform::default())
            .with(KeyboardMovementTag)
            .with(Speed(100.0))
            .build();

        // create eye attached to player.
        world
            .create_entity()
            .with(CompositeRenderable(
                Rectangle {
                    color: Color::yellow(),
                    rect: [-10.0, -10.0, 20.0, 20.0].into(),
                }
                .into(),
            ))
            .with(CompositeTransform::translation((-20.0).into()))
            .with(Parent(player))
            .build();

        // create hint text.
        world
            .create_entity()
            .with(CompositeRenderable(
                Text {
                    color: Color::white(),
                    font: "Verdana".into(),
                    align: TextAlign::Center,
                    text: "Use WSAD to move".into(),
                    position: 0.0.into(),
                    size: 24.0,
                }
                .into(),
            ))
            .with(CompositeTransform::translation([0.0, 100.0].into()))
            .with(CompositeRenderDepth(-1.0))
            .build();
    }
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    // Application build phase - install all systems and resources and setup them.
    let app = App::build()
        // install core module assets managment.
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                // register assets protocols from composite renderer module.
                oxygengine::composite_renderer::protocols_installer(assets);
            }),
        )
        // install input managment.
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            // register input devices.
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            // input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            // map input axes and triggers to devices.
            input.map_axis("move-up", "keyboard", "KeyW");
            input.map_axis("move-down", "keyboard", "KeyS");
            input.map_axis("move-left", "keyboard", "KeyA");
            input.map_axis("move-right", "keyboard", "KeyD");
            // input.map_axis("mouse-x", "mouse", "x");
            // input.map_axis("mouse-y", "mouse", "y");
            // input.map_trigger("mouse-left", "mouse", "left");
        })
        // install composite renderer.
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"), // canvas target.
                RenderState::new(Some(Color::black())),
            ),
        )
        .with_system(KeyboardMovementSystem, "keyboard_movement", &[])
        .build(GameState, WebAppTimer::default());

    // Application run phase - spawn runner that ticks our app.
    AppRunner::new(app).run::<WebAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
