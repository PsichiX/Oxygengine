extern crate byteorder;
extern crate oxygengine_core as core;
extern crate oxygengine_network as network;

pub mod client;
pub mod server;

pub mod prelude {
    pub use crate::client::*;
    pub use crate::server::*;
}
