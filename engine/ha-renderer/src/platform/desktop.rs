#![cfg(feature = "desktop")]

use crate::platform::{HaPlatformInterface, HaPlatformInterfaceProcessResult};
use glow::Context;
use oxygengine_backend_desktop::app::DesktopContextWrapper;
use std::sync::Arc;

#[derive(Debug)]
pub struct DesktopPlatformInterface {
    context: Option<Context>,
    context_wrapper: Arc<DesktopContextWrapper>,
    context_lost: bool,
}

// TODO: this is a hack and works only if single threaded or pinned render thread.
unsafe impl Send for DesktopPlatformInterface {}
unsafe impl Sync for DesktopPlatformInterface {}

impl DesktopPlatformInterface {
    pub fn with_context_wrapper(context_wrapper: Arc<DesktopContextWrapper>) -> Self {
        Self {
            context: unsafe {
                Some(Context::from_loader_function(|name| {
                    context_wrapper.get_proc_address(name) as *const _
                }))
            },
            context_wrapper,
            context_lost: false,
        }
    }
}

impl HaPlatformInterface for DesktopPlatformInterface {
    fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    fn screen_size(&self) -> (usize, usize) {
        let size = self.context_wrapper.window().inner_size();
        (size.width as _, size.height as _)
    }

    fn maintain(&mut self) -> HaPlatformInterfaceProcessResult<'_> {
        let mut result = HaPlatformInterfaceProcessResult::default();
        if self.context_lost {
            result.context_lost = std::mem::take(&mut self.context);
        }
        result
    }

    fn lose_context(&mut self) {
        self.context_lost = true;
    }
}
