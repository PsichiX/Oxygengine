#[cfg(feature = "parallel")]
use crate::ecs::System;
use crate::ecs::{
    pipeline::{PipelineEngine, PipelineGraph, PipelineGraphSystem},
    Universe,
};
#[cfg(feature = "parallel")]
use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[cfg(feature = "parallel")]
#[derive(Debug, Default)]
struct Access {
    read: usize,
    write: bool,
}

#[cfg(feature = "parallel")]
impl Access {
    pub fn can_read(&self) -> bool {
        !self.write
    }

    pub fn can_write(&self) -> bool {
        !self.write && self.read == 0
    }

    pub fn acquire_read(&mut self) {
        if self.can_read() {
            self.read += 1;
        }
    }

    pub fn acquire_write(&mut self) {
        if self.can_write() {
            self.write = true;
        }
    }

    pub fn release_read(&mut self) {
        self.read = self.read.checked_sub(1).unwrap_or_default();
    }

    pub fn release_write(&mut self) {
        self.write = false;
    }
}

#[cfg(feature = "parallel")]
struct Message {
    universe: &'static Universe,
    system: System,
}

#[cfg(feature = "parallel")]
type Notifier = Arc<(Mutex<bool>, Condvar)>;

#[cfg(feature = "parallel")]
struct Worker {
    sender: Sender<Option<Message>>,
    handle: JoinHandle<()>,
}

pub struct JobsPipelineEngine {
    #[cfg(feature = "parallel")]
    workers: Vec<Worker>,
    #[cfg(feature = "parallel")]
    notifier: Notifier,
    #[cfg(feature = "parallel")]
    receiver: Receiver<(usize, Option<(System, Duration)>)>,
    #[cfg(feature = "parallel")]
    systems_last_duration: RefCell<Vec<Duration>>,
    #[cfg(feature = "parallel")]
    systems_preferred_worker: RefCell<Vec<Option<usize>>>,
    pub(crate) systems: Vec<PipelineGraphSystem>,
}

unsafe impl Send for JobsPipelineEngine {}
unsafe impl Sync for JobsPipelineEngine {}

impl Default for JobsPipelineEngine {
    #[cfg(not(feature = "parallel"))]
    fn default() -> Self {
        Self::new(1)
    }

    #[cfg(feature = "parallel")]
    fn default() -> Self {
        Self::new(rayon::current_num_threads())
    }
}

impl JobsPipelineEngine {
    #[cfg(not(feature = "parallel"))]
    pub fn new(_jobs_count: usize) -> Self {
        Self {
            systems: Default::default(),
        }
    }

    #[cfg(feature = "parallel")]
    pub fn new(jobs_count: usize) -> Self {
        #[allow(clippy::mutex_atomic)]
        let notifier = Arc::new((Mutex::new(false), Condvar::new()));
        let (sender, receiver) = channel();
        let workers = Self::build_workers(notifier.clone(), sender, jobs_count);
        Self {
            workers,
            notifier,
            receiver,
            systems_last_duration: Default::default(),
            systems_preferred_worker: Default::default(),
            systems: Default::default(),
        }
    }

    #[cfg(feature = "parallel")]
    fn build_workers(
        notifier: Notifier,
        sender: Sender<(usize, Option<(System, Duration)>)>,
        mut jobs_count: usize,
    ) -> Vec<Worker> {
        jobs_count = jobs_count.max(1);
        (0..jobs_count)
            .into_iter()
            .map(|index| {
                let (my_sender, receiver) = channel();
                let notifier = Arc::clone(&notifier);
                let sender = sender.clone();
                let handle = std::thread::spawn(move || {
                    let (lock, cvar) = &*notifier;
                    while let Ok(msg) = receiver.recv() {
                        if let Some(Message { universe, system }) = msg {
                            let timer = Instant::now();
                            #[allow(mutable_transmutes)]
                            #[allow(clippy::transmute_ptr_to_ptr)]
                            system(unsafe { std::mem::transmute(universe) });
                            let _ = sender.send((index, Some((system, timer.elapsed()))));
                            let mut busy = lock.lock().unwrap();
                            *busy = false;
                            cvar.notify_all();
                        } else {
                            break;
                        }
                    }
                    let _ = sender.send((index, None));
                    let mut busy = lock.lock().unwrap();
                    *busy = false;
                    cvar.notify_all();
                });
                Worker {
                    sender: my_sender,
                    handle,
                }
            })
            .collect::<Vec<_>>()
    }

    #[cfg(feature = "parallel")]
    fn find_system_to_run(
        systems_left: &[usize],
        systems: &[PipelineGraphSystem],
        resources: &HashMap<TypeId, Access>,
        worker_index: usize,
        systems_preferred_worker: &[Option<usize>],
    ) -> Option<usize> {
        for index in systems_left {
            if let Some(index) = systems_preferred_worker[*index] {
                if worker_index != index {
                    continue;
                }
            }
            let data = &systems[*index];
            let can_read = data.reads.iter().all(|id| {
                resources
                    .get(id)
                    .map(|access| access.can_read())
                    .unwrap_or(true)
            });
            let can_write = data.writes.iter().all(|id| {
                resources
                    .get(id)
                    .map(|access| access.can_write())
                    .unwrap_or(true)
            });
            if can_read && can_write {
                return Some(*index);
            }
        }
        None
    }
}

impl PipelineEngine for JobsPipelineEngine {
    fn setup(&mut self, graph: PipelineGraph) {
        match graph {
            PipelineGraph::System(system) => {
                #[cfg(feature = "parallel")]
                self.systems_last_duration
                    .borrow_mut()
                    .push(Default::default());
                #[cfg(feature = "parallel")]
                self.systems_preferred_worker.borrow_mut().push(None);
                self.systems.push(system);
            }
            PipelineGraph::Sequence(list) | PipelineGraph::Parallel(list) => {
                for item in list {
                    self.setup(item);
                }
            }
        }
    }

    fn run(&self, universe: &mut Universe) {
        #[cfg(not(feature = "parallel"))]
        {
            for system in &self.systems {
                (system.system)(universe);
            }
        }
        #[cfg(feature = "parallel")]
        {
            if self.workers.len() <= 1 {
                for system in &self.systems {
                    (system.system)(universe);
                }
                return;
            }
            let mut systems_last_duration = self.systems_last_duration.borrow_mut();
            let mut systems_preferred_worker = self.systems_preferred_worker.borrow_mut();
            let mut systems_left = (0..self.systems.len()).into_iter().collect::<Vec<_>>();
            let mut load = vec![(false, Duration::default()); self.workers.len()];
            let mut sorted_load = (0..self.workers.len()).into_iter().collect::<Vec<_>>();
            let mut resources = self
                .systems
                .iter()
                .flat_map(|s| s.reads.iter().chain(s.writes.iter()))
                .map(|id| (*id, Access::default()))
                .collect::<HashMap<_, _>>();
            loop {
                let (lock, cvar) = &*self.notifier;
                let mut guard = cvar
                    .wait_while(lock.lock().unwrap(), |pending| *pending)
                    .unwrap();
                if systems_left.is_empty() {
                    break;
                }
                while let Ok((worker_index, duration)) = self.receiver.try_recv() {
                    let load = &mut load[worker_index];
                    load.0 = false;
                    if let Some((system, duration)) = duration {
                        load.1 += duration;
                        let found = self.systems.iter().position(|s| {
                            let a = s.system as *const ();
                            let b = system as *const ();
                            a == b
                        });
                        if let Some(system_index) = found {
                            systems_last_duration[system_index] = duration;
                            if self.systems[system_index].lock_on_single_thread {
                                systems_preferred_worker[system_index] = Some(worker_index);
                            }
                            for id in &self.systems[system_index].reads {
                                if let Some(access) = resources.get_mut(id) {
                                    access.release_read();
                                }
                            }
                            for id in &self.systems[system_index].writes {
                                if let Some(access) = resources.get_mut(id) {
                                    access.release_write();
                                }
                            }
                        }
                    }
                }
                sorted_load.sort_by(|a, b| load[*a].1.cmp(&load[*b].1));
                systems_left.sort_by(|a, b| {
                    self.systems[*a]
                        .layer
                        .cmp(&self.systems[*b].layer)
                        .then_with(|| systems_last_duration[*a].cmp(&systems_last_duration[*b]))
                });
                *guard = true;
                let layer = self.systems[*systems_left.first().unwrap()].layer;
                let mut chunk_size = systems_left.iter().fold(0, |a, v| {
                    a + if self.systems[*v].layer == layer {
                        1
                    } else {
                        0
                    }
                });
                for index in sorted_load.iter().copied() {
                    let mut load = &mut load[index];
                    if load.0 {
                        continue;
                    }
                    let worker = &self.workers[index];
                    let found = Self::find_system_to_run(
                        &systems_left[..chunk_size],
                        &self.systems,
                        &resources,
                        index,
                        &systems_preferred_worker,
                    );
                    if let Some(index) = found {
                        #[allow(mutable_transmutes)]
                        #[allow(clippy::transmute_ptr_to_ptr)]
                        let universe = unsafe { std::mem::transmute(&mut *universe) };
                        let msg = Message {
                            universe,
                            system: self.systems[index].system,
                        };
                        if worker.sender.send(Some(msg)).is_ok() {
                            for id in &self.systems[index].reads {
                                if let Some(access) = resources.get_mut(id) {
                                    access.acquire_read();
                                }
                            }
                            for id in &self.systems[index].writes {
                                if let Some(access) = resources.get_mut(id) {
                                    access.acquire_write();
                                }
                            }
                            if let Some(index) = systems_left.iter().position(|i| *i == index) {
                                systems_left.swap_remove(index);
                            }
                            load.0 = true;
                            chunk_size -= 1;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "parallel")]
impl Drop for JobsPipelineEngine {
    fn drop(&mut self) {
        for worker in std::mem::take(&mut self.workers) {
            let _ = worker.sender.send(None);
            let _ = worker.handle.join();
        }
    }
}
