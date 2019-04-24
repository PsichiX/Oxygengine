#[cfg(feature = "web")]
pub type PlatformAppRunner = crate::backend::web::app::WebAppRunner;
#[cfg(not(feature = "web"))]
pub type PlatformAppRunner = crate::core::app::SyncAppRunner;

#[cfg(feature = "web")]
#[cfg(feature = "composite-renderer")]
pub type PlatformCompositeRenderer = crate::backend::web::WebCompositeRenderer;

#[cfg(feature = "web")]
pub type PlatformFetchEngine = crate::backend::web::fetch::engines::web::WebFetchEngine;
#[cfg(not(feature = "web"))]
pub type PlatformFetchEngine = crate::core::fetch::engines::fs::FsFetchEngine;

#[cfg(feature = "web")]
pub type PlatformAppTimer = crate::backend::web::app::WebAppTimer;
#[cfg(not(feature = "web"))]
pub type PlatformAppTimer = crate::core::app::StandardAppTimer;
