pub struct DoOnDrop {
    executor: Box<FnMut()>,
}

impl DoOnDrop {
    pub fn new<F>(executor: F) -> Self
    where
        F: 'static + FnMut(),
    {
        Self {
            executor: Box::new(executor),
        }
    }
}

impl Drop for DoOnDrop {
    fn drop(&mut self) {
        (self.executor)();
    }
}
