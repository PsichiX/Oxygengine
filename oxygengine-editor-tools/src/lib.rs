#[cfg(feature = "web")]
pub mod web;

pub mod prelude {
    #[cfg(feature = "web")]
    pub use crate::web::*;
}
