extern crate oxygengine_core as core;

pub mod app;
pub mod closure;
pub mod fetch;
pub mod log;
pub mod storage;

pub mod prelude {
    pub use crate::{app::*, closure::*, fetch::*, log::*, storage::*};
}
