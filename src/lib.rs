#[cfg(feature = "web")]
extern crate oxygengine_backend_web;
#[cfg(feature = "composite-renderer")]
extern crate oxygengine_composite_renderer;
#[cfg(feature = "web")]
#[cfg(feature = "composite-renderer")]
extern crate oxygengine_composite_renderer_backend_web;
extern crate oxygengine_core;
#[cfg(feature = "input")]
extern crate oxygengine_input;
#[cfg(feature = "web")]
#[cfg(feature = "input")]
extern crate oxygengine_input_device_web;

pub mod core {
    pub use oxygengine_core::*;
}
#[cfg(feature = "input")]
pub mod input {
    pub use oxygengine_input::*;
}
pub mod backend {
    #[cfg(feature = "web")]
    pub mod web {
        pub use oxygengine_backend_web::*;
        #[cfg(feature = "composite-renderer")]
        pub use oxygengine_composite_renderer_backend_web::*;
        #[cfg(feature = "input")]
        pub use oxygengine_input_device_web::*;
    }
}
#[cfg(feature = "composite-renderer")]
pub mod composite_renderer {
    pub use oxygengine_composite_renderer::*;
}

pub mod prelude {
    #[cfg(feature = "web")]
    pub use oxygengine_backend_web::prelude::*;
    #[cfg(feature = "composite-renderer")]
    pub use oxygengine_composite_renderer::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "composite-renderer")]
    pub use oxygengine_composite_renderer_backend_web::prelude::*;
    pub use oxygengine_core::prelude::*;
    #[cfg(feature = "input")]
    pub use oxygengine_input::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "input")]
    pub use oxygengine_input_device_web::prelude::*;
}
