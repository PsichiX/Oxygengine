// reexport core macros.
pub use oxygengine_core::{error, info, log, warn};

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
        #[cfg(feature = "audio")]
        pub use oxygengine_audio_backend_web::*;
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
    #[cfg(feature = "native")]
    pub mod desktop {
        #[cfg(feature = "network")]
        pub use oxygengine_network_backend_native::*;
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
#[cfg(feature = "navigation")]
pub mod navigation {
    pub use oxygengine_navigation::*;
}
#[cfg(feature = "audio")]
pub mod audio {
    pub use oxygengine_audio::*;
}
#[cfg(feature = "physics-2d")]
pub mod physics_2d {
    pub use oxygengine_physics_2d::*;
}
#[cfg(feature = "physics-2d")]
#[cfg(feature = "composite-renderer")]
#[cfg(feature = "integration-physics-2d-composite-renderer")]
pub mod integration_physics_2d_composite_renderer {
    pub use oxygengine_integration_p2d_cr::*;
}
pub mod script {
    pub mod web {
        #[cfg(feature = "script-web")]
        pub use oxygengine_script_web::*;
    }
}

pub mod prelude {
    #[cfg(feature = "audio")]
    pub use oxygengine_audio::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "audio")]
    pub use oxygengine_audio_backend_web::prelude::*;
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
    #[cfg(feature = "physics-2d")]
    #[cfg(feature = "composite-renderer")]
    #[cfg(feature = "integration-physics-2d-composite-renderer")]
    pub use oxygengine_integration_p2d_cr::prelude::*;
    #[cfg(feature = "navigation")]
    pub use oxygengine_navigation::prelude::*;
    #[cfg(feature = "network")]
    pub use oxygengine_network::prelude::*;
    #[cfg(feature = "desktop")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_desktop::prelude::*;
    #[cfg(feature = "native")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_native::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "network")]
    pub use oxygengine_network_backend_web::prelude::*;
    #[cfg(feature = "physics-2d")]
    pub use oxygengine_physics_2d::prelude::*;
    #[cfg(feature = "procedural")]
    pub use oxygengine_procedural::prelude::*;
    #[cfg(feature = "web")]
    #[cfg(feature = "script-web")]
    pub use oxygengine_script_web::prelude::*;
    pub use oxygengine_utils::prelude::*;
}
