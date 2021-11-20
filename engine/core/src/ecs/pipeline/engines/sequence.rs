use crate::ecs::{
    pipeline::{PipelineEngine, PipelineGraph},
    System, Universe,
};
#[derive(Default, Clone)]
pub struct SequencePipelineEngine {
    pub(crate) systems: Vec<System>,
}

impl PartialEq for SequencePipelineEngine {
    fn eq(&self, other: &Self) -> bool {
        if self.systems.len() != other.systems.len() {
            return false;
        }
        for (a, b) in self.systems.iter().zip(other.systems.iter()) {
            let a = *a as *const ();
            let b = *b as *const ();
            if a != b {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Debug for SequencePipelineEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequencePipelineEngine")
            .field(
                "systems",
                &self
                    .systems
                    .iter()
                    .map(|s| format!("{:p}", *s as *const ()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl PipelineEngine for SequencePipelineEngine {
    fn setup(&mut self, graph: PipelineGraph) {
        match graph {
            PipelineGraph::System(system) => self.systems.push(system.system),
            PipelineGraph::Sequence(list) | PipelineGraph::Parallel(list) => {
                for item in list {
                    self.setup(item);
                }
            }
        }
    }

    fn run(&self, universe: &mut Universe) {
        for system in &self.systems {
            system(universe);
        }
    }
}
