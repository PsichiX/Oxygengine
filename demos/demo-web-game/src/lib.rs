use crate::{
    resources::turn::TurnManager,
    states::loading::LoadingState,
    systems::{
        bonuses::BonusesSystem, camera_control::CameraControlSystem,
        destruction::DestructionSystem, follow::FollowSystem, game::GameSystem,
        health::HealthSystem, player_control::PlayerControlSystem, turn::TurnSystem,
    },
};
use oxygengine::prelude::*;
use wasm_bindgen::prelude::*;

mod components;
mod resources;
mod states;
mod systems;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    logger_setup(WebLogger);

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                oxygengine::composite_renderer::protocols_installer(assets);
                oxygengine::audio::protocols_installer(assets);
            }),
        )
        .with_bundle(oxygengine::input::bundle_installer, |input| {
            input.register(WebKeyboardInputDevice::new(get_event_target_document()));
            input.register(WebMouseInputDevice::new(get_event_target_by_id("screen")));
            input.map_trigger("fire", "keyboard", "Space");
            input.map_axis("move-up", "keyboard", "KeyW");
            input.map_axis("move-down", "keyboard", "KeyS");
            input.map_axis("move-left", "keyboard", "KeyA");
            input.map_axis("move-right", "keyboard", "KeyD");
            // input.map_axis("mouse-x", "mouse", "x");
            // input.map_axis("mouse-y", "mouse", "y");
            input.map_trigger("mouse-left", "mouse", "left");
        })
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state(
                get_canvas_by_id("screen"),
                RenderState::default().image_source_inner_margin(0.5),
            ),
        )
        .with_bundle(oxygengine::audio::bundle_installer, WebAudio::default())
        .with_bundle(
            oxygengine::physics_2d::bundle_installer,
            (
                Vector::new(0.0, 0.0),
                Physics2dWorldSimulationMode::FixedTimestepMaxIterations(3),
            ),
        )
        .with_bundle(
            oxygengine::integration_physics_2d_composite_renderer::bundle_installer,
            (),
        )
        .with_resource(TurnManager::default())
        .with_system(FollowSystem, "follow", &[])
        .with_system(HealthSystem, "health", &[])
        .with_system(BonusesSystem, "bonuses", &[])
        .with_system(PlayerControlSystem, "player-control", &[])
        .with_system(DestructionSystem, "destruction", &[])
        .with_system(CameraControlSystem, "camera-control", &[])
        .with_system(TurnSystem, "turns", &[])
        .with_system(GameSystem, "game", &[])
        .build(LoadingState::default(), WebAppTimer::default());

    AppRunner::new(app).run(WebAppRunner)?;

    Ok(())
}
