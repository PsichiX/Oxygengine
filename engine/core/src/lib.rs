#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pest_derive;

#[macro_use]
pub mod log;
pub mod app;
pub mod assets;
pub mod error;
pub mod fetch;
pub mod prefab;
pub mod state;
#[macro_use]
pub mod localization;
pub mod ecs;
pub mod jobs;
pub mod storage;

#[cfg(test)]
mod tests;

pub mod id {
    pub use typid::*;
}

pub mod ignite {
    pub use oxygengine_ignite_types as types;
}

pub mod prelude {
    pub use crate::{
        app::*,
        assets::{
            asset::*,
            asset_pack_preloader::*,
            assets_preloader::*,
            database::*,
            protocol::*,
            protocols::{
                binary::*, localization::*, pack::*, prefab::*, set::*, text::*, yaml::*, *,
            },
            system::*,
            *,
        },
        ecs::{
            components::*,
            hierarchy::*,
            life_cycle::*,
            pipeline::{
                engines::{closure::*, default::*, jobs::*, sequence::*, *},
                *,
            },
            *,
        },
        fetch::{
            engines::{map::*, *},
            *,
        },
        id::*,
        jobs::*,
        localization::*,
        log::*,
        prefab::*,
        state::*,
        storage::{
            engines::{map::*, *},
            *,
        },
        Scalar, *,
    };
    #[cfg(not(feature = "web"))]
    pub use crate::{fetch::engines::fs::*, storage::engines::fs::*};
}

#[cfg(feature = "scalar64")]
pub type Scalar = f64;
#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;

pub use hecs::Bundle;
pub use oxygengine_ignite_derive::{ignite_proxy, Ignite};
