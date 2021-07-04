pub mod engines;

use crate::ecs::{AccessType, System, Universe};
pub use hecs::*;
use std::{any::TypeId, collections::HashSet, marker::PhantomData};
use typid::ID;

pub type PipelineID = ID<PhantomData<dyn PipelineEngine + Send + Sync>>;

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineBuilderError {
    DependencyNotFound(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipelineLayer {
    Pre,
    Main,
    Post,
}

impl Default for PipelineLayer {
    fn default() -> Self {
        Self::Main
    }
}

pub trait PipelineBuilder: Sized {
    fn add_system_on_layer<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<(), PipelineBuilderError>;

    fn add_system<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
    ) -> Result<(), PipelineBuilderError> {
        self.add_system_on_layer::<AT>(name, system, dependencies, PipelineLayer::Main)
    }

    fn with_system_on_layer<AT: AccessType>(
        mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<Self, PipelineBuilderError> {
        self.add_system_on_layer::<AT>(name, system, dependencies, layer)?;
        Ok(self)
    }

    fn with_system<AT: AccessType>(
        self,
        name: &str,
        system: System,
        dependencies: &[&str],
    ) -> Result<Self, PipelineBuilderError> {
        self.with_system_on_layer::<AT>(name, system, dependencies, PipelineLayer::Main)
    }

    fn graph(self) -> PipelineGraph;

    fn build<T>(self) -> T
    where
        T: PipelineEngine + Default,
    {
        self.build_with_engine(T::default())
    }

    fn build_with_engine<T>(self, mut engine: T) -> T
    where
        T: PipelineEngine,
    {
        engine.setup(self.graph());
        engine
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PipelineBuilderMeta {
    name: String,
    system: PipelineGraphSystem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParallelPipelineBuilder {
    parallel_jobs: usize,
    systems_pre: Vec<Vec<PipelineBuilderMeta>>,
    systems_main: Vec<Vec<PipelineBuilderMeta>>,
    systems_post: Vec<Vec<PipelineBuilderMeta>>,
}

impl Default for ParallelPipelineBuilder {
    #[cfg(not(feature = "parallel"))]
    fn default() -> Self {
        Self::new(1)
    }

    #[cfg(feature = "parallel")]
    fn default() -> Self {
        Self::new(rayon::current_num_threads())
    }
}

impl ParallelPipelineBuilder {
    pub fn new(parallel_jobs: usize) -> Self {
        Self {
            parallel_jobs: parallel_jobs.max(1),
            systems_pre: Default::default(),
            systems_main: Default::default(),
            systems_post: Default::default(),
        }
    }
}

impl PipelineBuilder for ParallelPipelineBuilder {
    fn add_system_on_layer<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<(), PipelineBuilderError> {
        let systems = match layer {
            PipelineLayer::Pre => &mut self.systems_pre,
            PipelineLayer::Main => &mut self.systems_main,
            PipelineLayer::Post => &mut self.systems_post,
        };
        for dep in dependencies {
            if !systems
                .iter()
                .any(|g| g.iter().any(|meta| meta.name.as_str() == *dep))
            {
                return Err(PipelineBuilderError::DependencyNotFound(dep.to_string()));
            }
        }
        let (reads, writes) = AT::get_types();
        if self.parallel_jobs == 1 {
            systems.push(vec![PipelineBuilderMeta {
                name: name.to_owned(),
                system: PipelineGraphSystem {
                    system,
                    reads,
                    writes,
                    layer,
                },
            }]);
            return Ok(());
        }
        let mut dependencies_left = dependencies.iter().copied().collect::<HashSet<_>>();
        for group in systems.iter_mut() {
            if !dependencies_left.is_empty() {
                for meta in group {
                    dependencies_left.remove(meta.name.as_str());
                }
            } else if group.len() < self.parallel_jobs
                && group
                    .iter()
                    .all(|meta| meta.system.writes.is_disjoint(&writes))
            {
                group.push(PipelineBuilderMeta {
                    name: name.to_owned(),
                    system: PipelineGraphSystem {
                        system,
                        reads,
                        writes,
                        layer,
                    },
                });
                return Ok(());
            }
        }
        systems.push(vec![PipelineBuilderMeta {
            name: name.to_owned(),
            system: PipelineGraphSystem {
                system,
                reads,
                writes,
                layer,
            },
        }]);
        Ok(())
    }

    fn graph(self) -> PipelineGraph {
        PipelineGraph::Sequence(
            self.systems_pre
                .into_iter()
                .map(|group| {
                    PipelineGraph::Parallel(
                        group
                            .into_iter()
                            .map(|meta| PipelineGraph::System(meta.system))
                            .collect(),
                    )
                })
                .chain(self.systems_main.into_iter().map(|group| {
                    PipelineGraph::Parallel(
                        group
                            .into_iter()
                            .map(|meta| PipelineGraph::System(meta.system))
                            .collect(),
                    )
                }))
                .chain(self.systems_post.into_iter().map(|group| {
                    PipelineGraph::Parallel(
                        group
                            .into_iter()
                            .map(|meta| PipelineGraph::System(meta.system))
                            .collect(),
                    )
                }))
                .collect(),
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LinearPipelineBuilder {
    systems_pre: Vec<PipelineBuilderMeta>,
    systems_main: Vec<PipelineBuilderMeta>,
    systems_post: Vec<PipelineBuilderMeta>,
}

impl PipelineBuilder for LinearPipelineBuilder {
    fn add_system_on_layer<AT: AccessType>(
        &mut self,
        name: &str,
        system: System,
        dependencies: &[&str],
        layer: PipelineLayer,
    ) -> Result<(), PipelineBuilderError> {
        let systems = match layer {
            PipelineLayer::Pre => &mut self.systems_pre,
            PipelineLayer::Main => &mut self.systems_main,
            PipelineLayer::Post => &mut self.systems_post,
        };
        for dep in dependencies {
            if !systems.iter().any(|meta| meta.name.as_str() == *dep) {
                return Err(PipelineBuilderError::DependencyNotFound(dep.to_string()));
            }
        }
        let (reads, writes) = AT::get_types();
        systems.push(PipelineBuilderMeta {
            name: name.to_string(),
            system: PipelineGraphSystem {
                system,
                reads,
                writes,
                layer,
            },
        });
        Ok(())
    }

    fn graph(self) -> PipelineGraph {
        PipelineGraph::Sequence(
            self.systems_pre
                .into_iter()
                .map(|meta| PipelineGraph::System(meta.system))
                .chain(
                    self.systems_main
                        .into_iter()
                        .map(|meta| PipelineGraph::System(meta.system)),
                )
                .chain(
                    self.systems_post
                        .into_iter()
                        .map(|meta| PipelineGraph::System(meta.system)),
                )
                .collect(),
        )
    }
}

#[derive(Clone)]
pub struct PipelineGraphSystem {
    pub system: System,
    pub reads: HashSet<TypeId>,
    pub writes: HashSet<TypeId>,
    pub layer: PipelineLayer,
}

impl PartialEq for PipelineGraphSystem {
    fn eq(&self, other: &Self) -> bool {
        let a = self.system as *const ();
        let b = other.system as *const ();
        a == b && self.reads == other.reads && self.writes == other.writes
    }
}

impl std::fmt::Debug for PipelineGraphSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineGraphSystem")
            .field("system", &format!("{:p}", self.system as *const ()))
            .field("reads", &self.reads)
            .field("writes", &self.writes)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineGraph {
    System(PipelineGraphSystem),
    Sequence(Vec<PipelineGraph>),
    Parallel(Vec<PipelineGraph>),
}

pub trait PipelineEngine {
    fn setup(&mut self, graph: PipelineGraph);
    fn run(&self, universe: &mut Universe);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::pipeline::{
        engines::{default::DefaultPipelineEngine, sequence::SequencePipelineEngine},
        LinearPipelineBuilder, ParallelPipelineBuilder,
    };

    macro_rules! types {
        () => (std::collections::HashSet::new());
        ( $($p:path),* ) => {
            {
                #[allow(unused_mut)]
                let mut result = std::collections::HashSet::new();
                $( result.insert(std::any::TypeId::of::<$p>()); )*
                result
            }
        }
    }

    #[test]
    fn test_pipeline_builder() {
        struct A;
        struct B;
        struct C;

        fn system_a(_: &mut Universe) {}
        fn system_b(_: &mut Universe) {}
        fn system_c(_: &mut Universe) {}

        let builder = ParallelPipelineBuilder::new(8)
            .with_system::<&mut A>("a", system_a, &[])
            .unwrap()
            .with_system::<&mut B>("b", system_b, &[])
            .unwrap()
            .with_system::<(&mut A, &mut B)>("c", system_c, &[])
            .unwrap()
            .with_system::<&mut C>("cc", system_c, &["a", "b"])
            .unwrap()
            .with_system::<()>("ccc", system_c, &[])
            .unwrap();
        assert_eq!(
            builder,
            ParallelPipelineBuilder {
                parallel_jobs: 8,
                systems_pre: vec![],
                systems_main: vec![
                    vec![
                        PipelineBuilderMeta {
                            name: "a".to_owned(),
                            system: PipelineGraphSystem {
                                system: system_a,
                                reads: types!(),
                                writes: types!(A),
                                layer: PipelineLayer::Main,
                            },
                        },
                        PipelineBuilderMeta {
                            name: "b".to_owned(),
                            system: PipelineGraphSystem {
                                system: system_b,
                                reads: types!(),
                                writes: types!(B),
                                layer: PipelineLayer::Main,
                            },
                        },
                        PipelineBuilderMeta {
                            name: "ccc".to_owned(),
                            system: PipelineGraphSystem {
                                system: system_c,
                                reads: types!(),
                                writes: types!(),
                                layer: PipelineLayer::Main,
                            },
                        },
                    ],
                    vec![
                        PipelineBuilderMeta {
                            name: "c".to_owned(),
                            system: PipelineGraphSystem {
                                system: system_c,
                                reads: types!(),
                                writes: types!(A, B),
                                layer: PipelineLayer::Main,
                            },
                        },
                        PipelineBuilderMeta {
                            name: "cc".to_owned(),
                            system: PipelineGraphSystem {
                                system: system_c,
                                reads: types!(),
                                writes: types!(C),
                                layer: PipelineLayer::Main,
                            },
                        },
                    ],
                ],
                systems_post: vec![],
            }
        );
        assert_eq!(
            builder.clone().graph(),
            PipelineGraph::Sequence(vec![
                PipelineGraph::Parallel(vec![
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_a,
                        reads: types!(),
                        writes: types!(A),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_b,
                        reads: types!(),
                        writes: types!(B),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(),
                        layer: PipelineLayer::Main,
                    }),
                ]),
                PipelineGraph::Parallel(vec![
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(A, B),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(C),
                        layer: PipelineLayer::Main,
                    }),
                ]),
            ])
        );
        assert_eq!(
            builder.clone().build::<SequencePipelineEngine>(),
            SequencePipelineEngine {
                systems: vec![system_a, system_b, system_c, system_c, system_c,],
            }
        );
        assert_eq!(
            builder.clone().build::<DefaultPipelineEngine>(),
            DefaultPipelineEngine {
                parallel: false,
                graph: Some(PipelineGraph::Sequence(vec![
                    PipelineGraph::Parallel(vec![
                        PipelineGraph::System(PipelineGraphSystem {
                            system: system_a,
                            reads: types!(),
                            writes: types!(A),
                            layer: PipelineLayer::Main,
                        }),
                        PipelineGraph::System(PipelineGraphSystem {
                            system: system_b,
                            reads: types!(),
                            writes: types!(B),
                            layer: PipelineLayer::Main,
                        }),
                        PipelineGraph::System(PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(),
                            layer: PipelineLayer::Main,
                        }),
                    ]),
                    PipelineGraph::Parallel(vec![
                        PipelineGraph::System(PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(A, B),
                            layer: PipelineLayer::Main,
                        }),
                        PipelineGraph::System(PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(C),
                            layer: PipelineLayer::Main,
                        }),
                    ]),
                ])),
            }
        );

        let builder = LinearPipelineBuilder::default()
            .with_system::<&mut A>("a", system_a, &[])
            .unwrap()
            .with_system::<&mut B>("b", system_b, &[])
            .unwrap()
            .with_system::<(&mut A, &mut B)>("c", system_c, &[])
            .unwrap()
            .with_system::<&mut C>("cc", system_c, &["a", "b"])
            .unwrap()
            .with_system::<()>("ccc", system_c, &[])
            .unwrap();
        assert_eq!(
            builder,
            LinearPipelineBuilder {
                systems_pre: vec![],
                systems_main: vec![
                    PipelineBuilderMeta {
                        name: "a".to_owned(),
                        system: PipelineGraphSystem {
                            system: system_a,
                            reads: types!(),
                            writes: types!(A),
                            layer: PipelineLayer::Main,
                        },
                    },
                    PipelineBuilderMeta {
                        name: "b".to_owned(),
                        system: PipelineGraphSystem {
                            system: system_b,
                            reads: types!(),
                            writes: types!(B),
                            layer: PipelineLayer::Main,
                        },
                    },
                    PipelineBuilderMeta {
                        name: "c".to_owned(),
                        system: PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(A, B),
                            layer: PipelineLayer::Main,
                        },
                    },
                    PipelineBuilderMeta {
                        name: "cc".to_owned(),
                        system: PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(C),
                            layer: PipelineLayer::Main,
                        },
                    },
                    PipelineBuilderMeta {
                        name: "ccc".to_owned(),
                        system: PipelineGraphSystem {
                            system: system_c,
                            reads: types!(),
                            writes: types!(),
                            layer: PipelineLayer::Main,
                        },
                    },
                ],
                systems_post: vec![],
            }
        );
        assert_eq!(
            builder.clone().graph(),
            PipelineGraph::Sequence(vec![
                PipelineGraph::System(PipelineGraphSystem {
                    system: system_a,
                    reads: types!(),
                    writes: types!(A),
                    layer: PipelineLayer::Main,
                }),
                PipelineGraph::System(PipelineGraphSystem {
                    system: system_b,
                    reads: types!(),
                    writes: types!(B),
                    layer: PipelineLayer::Main,
                }),
                PipelineGraph::System(PipelineGraphSystem {
                    system: system_c,
                    reads: types!(),
                    writes: types!(A, B),
                    layer: PipelineLayer::Main,
                }),
                PipelineGraph::System(PipelineGraphSystem {
                    system: system_c,
                    reads: types!(),
                    writes: types!(C),
                    layer: PipelineLayer::Main,
                }),
                PipelineGraph::System(PipelineGraphSystem {
                    system: system_c,
                    reads: types!(),
                    writes: types!(),
                    layer: PipelineLayer::Main,
                }),
            ])
        );
        assert_eq!(
            builder.clone().build::<SequencePipelineEngine>(),
            SequencePipelineEngine {
                systems: vec![system_a, system_b, system_c, system_c, system_c,],
            }
        );
        assert_eq!(
            builder.clone().build::<DefaultPipelineEngine>(),
            DefaultPipelineEngine {
                parallel: false,
                graph: Some(PipelineGraph::Sequence(vec![
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_a,
                        reads: types!(),
                        writes: types!(A),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_b,
                        reads: types!(),
                        writes: types!(B),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(A, B),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(C),
                        layer: PipelineLayer::Main,
                    }),
                    PipelineGraph::System(PipelineGraphSystem {
                        system: system_c,
                        reads: types!(),
                        writes: types!(),
                        layer: PipelineLayer::Main,
                    }),
                ])),
            }
        );
    }
}
