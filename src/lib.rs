pub mod core {
    pub use oxygengine_core::*;
}
pub mod utils {
    pub use oxygengine_utils::*;
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
    #[cfg(feature = "desktop")]
    pub mod desktop {
        #[cfg(feature = "network")]
        pub use oxygengine_network_backend_desktop::*;
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
#[cfg(feature = "procedural")]
pub mod procedural {
    pub use oxygengine_procedural::*;
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
    #[cfg(feature = "desktop")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_desktop::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_web::prelude::*;
    #[cfg(feature = "procedural")]
    pub use oxygengine_procedural::prelude::*;
    pub use oxygengine_utils::prelude::*;
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => ({
        $crate::core::log::logger_log($lvl, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)+)
        ))
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (log!($crate::core::log::Log::Info, $($arg)*))
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (log!($crate::core::log::Log::Warning, $($arg)*))
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (log!($crate::core::log::Log::Error, $($arg)*))
}
