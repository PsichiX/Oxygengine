#[cfg(feature = "parallel")]
extern crate rayon;
#[macro_use]
extern crate lazy_static;
extern crate typid;

#[macro_use]
pub mod log;

pub mod app;
pub mod assets;
pub mod error;
pub mod fetch;
pub mod hierarchy;
pub mod state;

#[cfg(test)]
mod tests;

pub mod id {
    pub use typid::*;
}

pub mod ecs {
    pub use specs::*;
}

pub mod prelude {
    pub use crate::{
        app::*, assets::prelude::*, assets::*, ecs::*, fetch::prelude::*, fetch::*, hierarchy::*,
        id::*, log::*, state::*,
    };
}
