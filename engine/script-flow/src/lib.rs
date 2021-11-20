extern crate oxygengine_core as core;

pub mod ast;
pub mod resource;
pub mod system;
pub mod vm;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::{ast::*, resource::*, system::*, vm::*};
}

use crate::system::{flow_script_system, FlowScriptSystemResources};
use core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    id::ID,
};

pub type Guid = ID<()>;

pub fn bundle_installer<PB>(builder: &mut AppBuilder<PB>, _: ()) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_system::<FlowScriptSystemResources>("flow", flow_script_system, &[])?;
    Ok(())
}
