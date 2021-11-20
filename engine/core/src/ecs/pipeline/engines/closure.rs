use crate::ecs::{
    pipeline::{PipelineEngine, PipelineGraph},
    Universe,
};

pub struct ClosurePipelineEngine {
    closure: Box<dyn Fn(&mut Universe)>,
}

impl ClosurePipelineEngine {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut Universe) + 'static,
    {
        Self {
            closure: Box::new(f),
        }
    }
}

impl PipelineEngine for ClosurePipelineEngine {
    fn setup(&mut self, _: PipelineGraph) {}

    fn run(&self, universe: &mut Universe) {
        (self.closure)(universe);
    }
}

impl<F> From<F> for ClosurePipelineEngine
where
    F: Fn(&mut Universe) + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}
