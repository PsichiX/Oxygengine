#[cfg(feature = "desktop")]
pub mod desktop;
#[cfg(feature = "web")]
pub mod web;

use glow::*;

#[derive(Debug, Default)]
pub struct HaPlatformInterfaceProcessResult<'a> {
    pub context_acquired: Option<&'a Context>,
    pub context_lost: Option<Context>,
    pub screen_resized: Option<(usize, usize)>,
}

pub trait HaPlatformInterface {
    fn context(&self) -> Option<&Context>;
    fn screen_size(&self) -> (usize, usize);
    fn maintain(&mut self) -> HaPlatformInterfaceProcessResult;
    fn lose_context(&mut self);
}

impl HaPlatformInterface for () {
    fn context(&self) -> Option<&Context> {
        None
    }

    fn screen_size(&self) -> (usize, usize) {
        (0, 0)
    }

    fn maintain(&mut self) -> HaPlatformInterfaceProcessResult {
        Default::default()
    }

    fn lose_context(&mut self) {}
}
