#[cfg(feature = "parallel")]
extern crate rayon;
extern crate serde;
extern crate shrev;
extern crate specs;
extern crate uuid;

pub mod app;
pub mod assets;
pub mod error;
pub mod fetch;
pub mod id;
pub mod state;
#[cfg(test)]
mod tests;

pub mod ecs {
    pub use specs::*;
}
pub mod events {
    pub use shrev::*;
}

pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::ecs::*;
    pub use crate::events::*;
    pub use crate::fetch::*;
    pub use crate::id::*;
    pub use crate::state::*;
}
