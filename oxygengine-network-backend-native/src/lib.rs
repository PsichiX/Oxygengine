extern crate oxygengine_core as core;
extern crate oxygengine_network as network;

#[cfg(test)]
mod tests;

pub mod client;
pub mod server;
mod utils;

pub mod prelude {
    pub use crate::client::*;
    pub use crate::server::*;
}
