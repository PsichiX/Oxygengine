mod asset_protocols;
mod components;
mod resources;
mod states;
mod systems;
mod ui;
mod utils;

use crate::{
    asset_protocols::{part::*, parts::*},
    components::{animated_background::*, spinbot::*},
    resources::{parts_registry::*, ui_bridge::*},
    states::loading::LoadingState,
    systems::{
        animated_background::*, parts_registry::*, physics_collisions::*, physics_movement::*,
    },
    utils::physics::*,
};
use oxygengine::prelude::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // initialize logger to see logs in web browser (debug only).
    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    // Application build phase - install all systems and resources and setup them.
    let app = App::build::<LinearPipelineBuilder>()
        // install core module assets managment.
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                // register assets loading error reporter that shows errors in console.
                #[cfg(debug_assertions)]
                assets.register_error_reporter(LoggerAssetsDatabaseErrorReporter);
                // register assets protocols from composite renderer module.
                oxygengine::composite_renderer::protocols_installer(assets);
                assets.register(PartAssetProtocol);
                assets.register(PartsAssetProtocol);
            }),
        )
        .unwrap()
        // install core module prefabs management.
        .with_bundle(oxygengine::core::prefab::bundle_installer, |prefabs| {
            // install composite renderer prefabs.
            oxygengine::composite_renderer::prefabs_installer(prefabs);
            // install UI prefabs.
            oxygengine::user_interface::prefabs_installer(prefabs);
            // install prefabs for integration between UI and composite rendering.
            oxygengine::integration_user_interface_composite_renderer::prefabs_installer(prefabs);
            // install 2d physics prefabs.
            oxygengine::physics_2d::prefabs_installer(prefabs);
            // install prefabs for integration between 2D physics and composite rendering.
            oxygengine::integration_physics_2d_composite_renderer::prefabs_installer(prefabs);
            // register game prefabs component factories.
            prefabs.register_component_factory::<SpinBot>("SpinBot");
            prefabs.register_component_factory::<AnimatedBackground>("AnimatedBackground");
        })
        .unwrap()
        // install input managment.
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            // register input devices.
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            // map input axes and triggers to devices.
            input.map_axis("pointer-x", "mouse", "x");
            input.map_axis("pointer-y", "mouse", "y");
            input.map_trigger("pointer-action", "mouse", "left");
            input.map_trigger("pointer-context", "mouse", "right");
            input.map_axis("a-move-up", "keyboard", "KeyW");
            input.map_axis("a-move-down", "keyboard", "KeyS");
            input.map_axis("a-move-left", "keyboard", "KeyA");
            input.map_axis("a-move-right", "keyboard", "KeyD");
            input.map_axis("a-power", "keyboard", "Space");
            input.map_axis("b-move-up", "keyboard", "KeyI");
            input.map_axis("b-move-down", "keyboard", "KeyK");
            input.map_axis("b-move-left", "keyboard", "KeyJ");
            input.map_axis("b-move-right", "keyboard", "KeyL");
            input.map_axis("b-power", "keyboard", "Enter");
        })
        .unwrap()
        // install composite renderer.
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"), // canvas target.
                RenderState::new(Some(hex_to_rgb(0x5082FF))).image_source_inner_margin(0.01),
            ),
        )
        .unwrap()
        // install UI support.
        .with_bundle(
            oxygengine::user_interface::bundle_installer,
            UserInterfaceBundleSetup::default().user_interface(
                UserInterface::new(ui::setup)
                    .with_pointer_axis("pointer-x", "pointer-y")
                    .with_pointer_trigger("pointer-action", "pointer-context"),
            ),
        )
        .unwrap()
        // install integration between UI and composite rendering.
        .with_bundle(
            oxygengine::integration_user_interface_composite_renderer::bundle_installer::<
                WebCompositeRenderer,
                _,
            >,
            (),
        )
        .unwrap()
        // install 2D physics with default gravity force vector.
        .with_bundle(
            oxygengine::physics_2d::bundle_installer,
            (
                Vector::default(),
                Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
            ),
        )
        .unwrap()
        // install integration between 2D physics and composite rendering.
        .with_bundle(
            oxygengine::integration_physics_2d_composite_renderer::bundle_installer,
            (),
        )
        .unwrap()
        // install web storage engine resource.
        .with_resource(WebStorageEngine)
        .with_resource(PartsRegistry::default())
        .with_resource(build_arena())
        .with_resource(web_app_params())
        .with_resource(UiBridge::default())
        .with_system::<PartsRegistrySystemResources>("parts-registry", parts_registry_system, &[])
        .unwrap()
        .with_system::<AnimatedBackgroundSystemResources>(
            "animated-background",
            animated_background_system,
            &[],
        )
        .unwrap()
        .with_system::<PhysicsMovementSystemResources>(
            "physics-movement",
            physics_movement_system,
            &[],
        )
        .unwrap()
        .with_system::<PhysicsCollisionsSystemResources>(
            "physics-collisions",
            physics_collisions_system,
            &["physics-movement"],
        )
        .unwrap()
        .build::<SequencePipelineEngine, _, _>(
            LoadingState::default(),
            (WebAppTimer::default(), Duration::from_millis(1000 / 60)),
        );

    // Application run phase - spawn runner that ticks our app.
    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}

fn hex_to_rgb(value: u32) -> Color {
    let b = value & 0xFF;
    let g = (value >> 8) & 0xFF;
    let r = (value >> 16) & 0xFF;
    Color::rgb(r as _, g as _, b as _)
}

fn build_arena() -> Arena {
    Arena {
        shape: ArenaShape::new(512.0, 30.0_f32.to_radians())
            .expect("Could not construct arena shape"),
    }
}
