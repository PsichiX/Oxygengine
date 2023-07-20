#[cfg(feature = "desktop")]
pub mod desktop;
#[cfg(feature = "web")]
pub mod web;

use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_input::prelude::*;

pub trait PrototypeApp {
    fn clear_color(self, value: Rgba) -> Self;
    fn sprite_filtering(self, value: ImageFiltering) -> Self;
    fn view_size(self, value: Scalar) -> Self;
    fn preload_asset(self, path: impl ToString) -> Self;
    fn input_mappings(self, mappings: InputMappings) -> Self;
    fn run(self);
}

pub(crate) struct BootState {
    next_state: Option<Box<dyn State>>,
    view_size: Scalar,
}

impl State for BootState {
    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        if let Some(state) = self.next_state.take() {
            let mut commands = universe.expect_resource_mut::<UniverseCommands>();
            commands.schedule(SpawnEntity::from_bundle((
                Name("camera".into()),
                HaCamera::default()
                    .with_projection(HaCameraProjection::Orthographic(HaCameraOrthographic {
                        scaling: HaCameraOrtographicScaling::FitToView(
                            self.view_size.into(),
                            false,
                        ),
                        centered: true,
                        ignore_depth_planes: false,
                    }))
                    .with_viewport(RenderTargetViewport::Full)
                    .with_pipeline(PipelineSource::Registry("prototype".to_owned())),
                HaDefaultCamera,
                HaTransform::default(),
            )));
            StateChange::Swap(state)
        } else {
            StateChange::Pop
        }
    }
}
