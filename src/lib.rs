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
        #[cfg(feature = "network")]
        pub use oxygengine_network_backend_web::*;
    }
}
#[cfg(feature = "composite-renderer")]
pub mod composite_renderer {
    pub use oxygengine_composite_renderer::*;
}
#[cfg(feature = "network")]
pub mod network {
    pub use oxygengine_network::*;
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
    #[cfg(feature = "network")]
    pub use oxygengine_network::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_web::prelude::*;
}
