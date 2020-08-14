#[cfg(feature = "parallel")]
extern crate rayon;
#[macro_use]
extern crate lazy_static;
extern crate typid;
#[macro_use]
extern crate pest_derive;

#[macro_use]
pub mod log;
pub mod app;
pub mod assets;
pub mod error;
pub mod fetch;
pub mod hierarchy;
pub mod prefab;
pub mod state;
#[macro_use]
pub mod localization;
pub mod storage;

#[cfg(test)]
mod tests;

pub mod id {
    pub use typid::*;
}

pub mod ecs {
    pub use shred::Resource;
    pub use shrev::*;
    pub use specs::*;
}

pub mod ignite {
    pub use oxygengine_ignite_types as types;
}

pub mod prelude {
    pub use crate::{
        app::*, assets::prelude::*, ecs::*, fetch::prelude::*, fetch::*, hierarchy::*, id::*,
        localization::*, log::*, prefab::*, state::*, storage::prelude::*, storage::*, Ignite,
        Scalar,
    };
}

#[cfg(feature = "scalar64")]
pub type Scalar = f64;
#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;

pub use oxygengine_ignite_derive::{ignite_proxy, Ignite};
