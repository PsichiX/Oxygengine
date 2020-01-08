extern crate oxygengine_core as core;
#[macro_use]
extern crate lazy_static;

pub mod component;
pub mod interface;
pub mod state;
pub mod system;
pub mod prelude {
    pub use crate::{component::*, interface::*, state::*, system::*};
}
