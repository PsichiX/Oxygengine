#[cfg(feature = "parallel")]
extern crate rayon;
extern crate serde;
extern crate specs;
extern crate specs_hierarchy;
extern crate uuid;
#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod log;

pub mod app;
pub mod assets;
pub mod error;
pub mod fetch;
pub mod hierarchy;
pub mod id;
pub mod state;

#[cfg(test)]
mod tests;

pub mod ecs {
    pub use specs::*;
}

pub mod prelude {
    pub use crate::{
        app::*, assets::prelude::*, assets::*, ecs::*, fetch::prelude::*, fetch::*, hierarchy::*,
        id::*, log::*, state::*,
    };
}
