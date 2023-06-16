use glutin::event::WindowEvent;

#[derive(Default)]
pub struct DesktopAppEvents {
    queue: Vec<WindowEvent<'static>>,
}

impl DesktopAppEvents {
    pub(crate) fn push(&mut self, event: WindowEvent<'static>) {
        self.queue.push(event);
    }

    pub(crate) fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &WindowEvent> {
        self.queue.iter()
    }
}
