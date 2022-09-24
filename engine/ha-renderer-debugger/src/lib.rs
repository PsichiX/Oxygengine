extern crate oxygengine_core as core;
extern crate oxygengine_editor_tools as editor;
extern crate oxygengine_ha_renderer as renderer;

pub mod request;
pub mod system;

pub mod prelude {
    pub use crate::{request::*, system::*};
}

use crate::system::{
    ha_renderer_debugger_system, HaRendererDebuggerSystemCache, HaRendererDebuggerSystemResources,
};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
};
use editor::simp::SimpChannel;

pub fn bundle_installer<PB, C>(
    builder: &mut AppBuilder<PB>,
    channel: C,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    C: SimpChannel + Send + Sync + 'static,
{
    #[cfg(debug_assertions)]
    builder.install_resource(HaRendererDebuggerSystemCache::new(channel));

    #[cfg(debug_assertions)]
    builder.install_system::<HaRendererDebuggerSystemResources<C>>(
        "renderer-debugger",
        ha_renderer_debugger_system::<C>,
        &[],
    )?;

    Ok(())
}
