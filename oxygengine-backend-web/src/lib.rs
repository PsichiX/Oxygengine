extern crate oxygengine_core as core;

pub mod app;
pub mod fetch;

pub mod prelude {
    pub use crate::app::*;
    pub use crate::fetch::*;
}
