// reexport core macros.
pub use oxygengine_core::{debug, error, info, log, warn};
#[cfg(feature = "oxygengine-input")]
pub use oxygengine_input::include_input_mappings;
#[cfg(feature = "oxygengine-nodes")]
pub use oxygengine_nodes::nodes_dispatch;
#[cfg(feature = "oxygengine-user-interface")]
pub use oxygengine_user_interface::{post_hooks, pre_hooks, unpack_named_slots, widget};

pub mod core {
    pub use oxygengine_core::*;
}
pub mod utils {
    pub use oxygengine_utils::*;
}
#[cfg(feature = "oxygengine-input")]
pub mod input {
    pub use oxygengine_input::*;
}
#[cfg(feature = "oxygengine-ha-renderer")]
pub mod ha_renderer {
    pub use oxygengine_ha_renderer::*;
}
#[cfg(feature = "oxygengine-network")]
pub mod network {
    pub use oxygengine_network::*;
}
#[cfg(feature = "oxygengine-procedural")]
pub mod procedural {
    pub use oxygengine_procedural::*;
}
#[cfg(feature = "oxygengine-prototype")]
pub mod prototype {
    pub use oxygengine_prototype::*;
}
#[cfg(feature = "oxygengine-nodes")]
pub mod nodes {
    pub use oxygengine_nodes::*;
}
#[cfg(feature = "oxygengine-navigation")]
pub mod navigation {
    pub use oxygengine_navigation::*;
}
#[cfg(feature = "oxygengine-audio")]
pub mod audio {
    pub use oxygengine_audio::*;
}
#[cfg(feature = "oxygengine-physics-2d")]
pub mod physics_2d {
    pub use oxygengine_physics_2d::*;
}
#[cfg(feature = "oxygengine-overworld")]
#[cfg(feature = "oxygengine-ha-renderer")]
#[cfg(feature = "oxygengine-integration-ow-ha")]
pub mod integration_overworld_ha_renderer {
    pub use oxygengine_integration_ow_ha::*;
}
#[cfg(feature = "oxygengine-user-interface")]
#[cfg(feature = "oxygengine-ha-renderer")]
#[cfg(feature = "oxygengine-integration-ui-ha")]
pub mod integration_user_interface_ha_renderer {
    pub use oxygengine_integration_ui_ha::*;
}
#[cfg(feature = "oxygengine-visual-novel")]
#[cfg(feature = "oxygengine-user-interface")]
#[cfg(feature = "oxygengine-integration-vn-ui")]
pub mod integration_visual_novel_user_interface {
    pub use oxygengine_integration_vn_ui::*;
}
#[cfg(feature = "oxygengine-ha-renderer")]
#[cfg(feature = "oxygengine-ha-renderer-debugger")]
pub mod ha_renderer_debugger {
    pub use oxygengine_ha_renderer_debugger::*;
}
#[cfg(feature = "oxygengine-visual-novel")]
pub mod visual_novel {
    pub use oxygengine_visual_novel::*;
}
#[cfg(feature = "oxygengine-overworld")]
pub mod overworld {
    pub use oxygengine_overworld::*;
}
#[cfg(feature = "oxygengine-ai")]
pub mod ai {
    pub use oxygengine_ai::*;
}
#[cfg(feature = "oxygengine-animation")]
pub mod animation {
    pub use oxygengine_animation::*;
}
#[cfg(feature = "oxygengine-user-interface")]
pub mod user_interface {
    pub use oxygengine_user_interface::*;
}
#[cfg(feature = "oxygengine-editor-tools")]
pub mod editor_tools {
    pub use oxygengine_editor_tools::*;
}

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    #[cfg(feature = "oxygengine-ai")]
    pub use oxygengine_ai::prelude::*;
    #[cfg(feature = "oxygengine-animation")]
    pub use oxygengine_animation::prelude::*;
    #[cfg(feature = "oxygengine-audio")]
    pub use oxygengine_audio::prelude::*;
    #[cfg(feature = "oxygengine-audio-backend-web")]
    pub use oxygengine_audio_backend_web::prelude::*;
    #[cfg(feature = "oxygengine-backend-desktop")]
    pub use oxygengine_backend_desktop::prelude::*;
    #[cfg(feature = "oxygengine-backend-web")]
    pub use oxygengine_backend_web::prelude::*;
    pub use oxygengine_core::{prelude::*, Scalar};
    #[cfg(feature = "oxygengine-editor-tools")]
    pub use oxygengine_editor_tools::prelude::*;
    #[cfg(feature = "oxygengine-editor-tools-backend-web")]
    pub use oxygengine_editor_tools_backend_web::prelude::*;
    #[cfg(feature = "oxygengine-ha-renderer")]
    pub use oxygengine_ha_renderer::prelude::*;
    #[cfg(feature = "oxygengine-ha-renderer")]
    #[cfg(feature = "oxygengine-ha-renderer-debugger")]
    pub use oxygengine_ha_renderer_debugger::prelude::*;
    #[cfg(feature = "oxygengine-input")]
    pub use oxygengine_input::prelude::*;
    #[cfg(feature = "oxygengine-input-device-desktop")]
    pub use oxygengine_input_device_desktop::prelude::*;
    #[cfg(feature = "oxygengine-input-device-web")]
    pub use oxygengine_input_device_web::prelude::*;
    #[cfg(feature = "oxygengine-integration-ow-ha")]
    pub use oxygengine_integration_ow_ha::prelude::*;
    #[cfg(feature = "oxygengine-integration-p2d-cr")]
    pub use oxygengine_integration_p2d_cr::prelude::*;
    #[cfg(feature = "oxygengine-integration-ui-cr")]
    pub use oxygengine_integration_ui_cr::prelude::*;
    #[cfg(feature = "oxygengine-integration-ui-ha")]
    pub use oxygengine_integration_ui_ha::prelude::*;
    #[cfg(feature = "oxygengine-integration-vn-ui")]
    pub use oxygengine_integration_vn_ui::prelude::*;
    #[cfg(feature = "oxygengine-navigation")]
    pub use oxygengine_navigation::prelude::*;
    #[cfg(feature = "oxygengine-network")]
    pub use oxygengine_network::prelude::*;
    #[cfg(feature = "oxygengine-network-backend-desktop")]
    pub use oxygengine_network_backend_desktop::prelude::*;
    #[cfg(feature = "oxygengine-network-backend-native")]
    pub use oxygengine_network_backend_native::prelude::*;
    #[cfg(feature = "oxygengine-network-backend-web")]
    pub use oxygengine_network_backend_web::prelude::*;
    #[cfg(feature = "oxygengine-nodes")]
    pub use oxygengine_nodes::*;
    #[cfg(feature = "oxygengine-overworld")]
    pub use oxygengine_overworld::prelude::*;
    #[cfg(feature = "oxygengine-physics-2d")]
    pub use oxygengine_physics_2d::prelude::*;
    #[cfg(feature = "oxygengine-procedural")]
    pub use oxygengine_procedural::prelude::*;
    #[cfg(feature = "oxygengine-prototype")]
    pub use oxygengine_prototype::prelude::*;
    #[cfg(feature = "oxygengine-user-interface")]
    pub use oxygengine_user_interface::prelude::*;
    pub use oxygengine_utils::prelude::*;
    #[cfg(feature = "oxygengine-visual-novel")]
    pub use oxygengine_visual_novel::prelude::*;
}
