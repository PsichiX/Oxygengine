#[cfg(feature = "parallel")]
extern crate rayon;
extern crate serde;
extern crate specs;
extern crate specs_hierarchy;
extern crate uuid;

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
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::ecs::*;
    pub use crate::fetch::*;
    pub use crate::hierarchy::*;
    pub use crate::id::*;
    pub use crate::state::*;
}
