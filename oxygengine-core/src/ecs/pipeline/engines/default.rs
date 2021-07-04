use crate::ecs::{
    pipeline::{PipelineEngine, PipelineGraph},
    Universe,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DefaultPipelineEngine {
    pub parallel: bool,
    pub(crate) graph: Option<PipelineGraph>,
}

impl DefaultPipelineEngine {
    pub fn with_parallel(mut self, mode: bool) -> Self {
        self.parallel = mode;
        self
    }

    fn run_node(node: &PipelineGraph, universe: &mut Universe) {
        match node {
            PipelineGraph::System(system) => (system.system)(universe),
            PipelineGraph::Sequence(list) | PipelineGraph::Parallel(list) => {
                for item in list {
                    Self::run_node(item, universe);
                }
            }
        }
    }

    #[cfg(feature = "parallel")]
    fn run_node_parallel(node: &PipelineGraph, universe: &Universe) {
        #[allow(mutable_transmutes)]
        match node {
            PipelineGraph::System(system) =>
            {
                #[allow(clippy::transmute_ptr_to_ptr)]
                (system.system)(unsafe { std::mem::transmute(universe) })
            }
            PipelineGraph::Sequence(list) => {
                for item in list {
                    Self::run_node_parallel(item, universe);
                }
            }
            PipelineGraph::Parallel(list) => {
                use rayon::prelude::*;
                if list.len() > 1 {
                    list.par_iter()
                        .for_each(|item| Self::run_node_parallel(item, universe));
                } else {
                    Self::run_node_parallel(list.first().unwrap(), universe);
                }
            }
        }
    }
}

impl PipelineEngine for DefaultPipelineEngine {
    fn setup(&mut self, graph: PipelineGraph) {
        self.graph = Some(graph);
    }

    fn run(&self, universe: &mut Universe) {
        #[cfg(not(feature = "parallel"))]
        {
            if let Some(node) = &self.graph {
                Self::run_node(node, universe);
            }
        }
        #[cfg(feature = "parallel")]
        {
            if let Some(node) = &self.graph {
                if self.parallel {
                    Self::run_node_parallel(node, universe);
                } else {
                    Self::run_node(node, universe);
                }
            }
        }
    }
}
