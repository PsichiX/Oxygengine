#[cfg(feature = "web")]
extern crate oxygengine_backend_web;
#[cfg(feature = "composite-renderer")]
extern crate oxygengine_composite_renderer;
#[cfg(feature = "web")]
extern crate oxygengine_composite_renderer_backend_web;
extern crate oxygengine_core;

pub mod platform;

pub mod core {
    pub use oxygengine_core::*;
}

pub mod backend {
    #[cfg(feature = "web")]
    pub mod web {
        pub use oxygengine_backend_web::*;
        #[cfg(feature = "composite-renderer")]
        pub use oxygengine_composite_renderer_backend_web::*;
    }
}

#[cfg(feature = "composite-renderer")]
pub mod composite_renderer {
    pub use oxygengine_composite_renderer::*;
}

pub mod prelude {
    #[cfg(feature = "web")]
    pub use crate::backend::web::*;
    #[cfg(feature = "composite-renderer")]
    pub use crate::composite_renderer::prelude::*;
    pub use crate::core::prelude::*;
    pub use crate::platform::*;
}
