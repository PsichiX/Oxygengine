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

use crate::system::FlowSystem;
use core::prelude::*;

pub type GUID = ID<()>;

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>, _: ()) {
    builder.install_system(FlowSystem, "flow", &[]);
}
