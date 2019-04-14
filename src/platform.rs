#[cfg(feature = "web")]
pub type PlatformAppRunner = crate::backend::web::app::WebAppRunner;
#[cfg(not(feature = "web"))]
pub type PlatformAppRunner = crate::core::app::SyncAppRunner;
#[cfg(feature = "web")]
#[cfg(feature = "composite-renderer")]
pub type PlatformCompositeRenderer = crate::backend::web::WebCompositeRenderer;
