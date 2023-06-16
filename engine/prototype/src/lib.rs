pub mod app;
pub mod constants;
pub mod gui;
pub mod materials;
pub mod resources;
pub mod systems;

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    #[cfg(feature = "web")]
    pub use crate::app::web::*;

    pub use crate::{
        app::*,
        constants::material_uniforms::*,
        gui::*,
        materials::*,
        resources::{audio_player::*, camera::*, renderables::*, *},
        systems::{audio_player::*, camera::*, render_prototype_stage::*, *},
    };
}

use crate::{
    resources::{audio_player::AudioPlayer, camera::Camera, renderables::Renderables},
    systems::{
        audio_player::{audio_player_system, AudioPlayerResources},
        camera::{camera_system, CameraSystemResources},
        render_prototype_stage::{
            ha_render_prototype_stage_system, HaRenderPrototypeStageSystemCache,
            HaRenderPrototypeStageSystemResources,
        },
    },
};
use oxygengine_core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
};

pub fn bundle_installer<PB, F>(
    builder: &mut AppBuilder<PB>,
    mut f: F,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    F: FnMut(&mut Renderables, &mut Camera),
{
    let mut renderables = Renderables::default();
    let mut camera = Camera::default();
    f(&mut renderables, &mut camera);

    builder.install_resource(renderables);
    builder.install_resource(camera);
    builder.install_resource(AudioPlayer::default());
    builder.install_resource(HaRenderPrototypeStageSystemCache::default());

    builder.install_system::<CameraSystemResources>("camera", camera_system, &[])?;
    builder.install_system::<HaRenderPrototypeStageSystemResources>(
        "render-prototype-stage",
        ha_render_prototype_stage_system,
        &[],
    )?;
    builder.install_system::<AudioPlayerResources>("audio-player", audio_player_system, &[])?;

    Ok(())
}
