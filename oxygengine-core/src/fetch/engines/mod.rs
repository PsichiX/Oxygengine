#[cfg(not(feature = "web"))]
pub mod fs;
pub mod map;

pub mod prelude {
    #[cfg(not(feature = "web"))]
    pub use super::fs::*;
    pub use super::map::*;
}
