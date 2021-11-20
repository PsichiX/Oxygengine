use std::iter::FromIterator;
#[cfg(feature = "parallel")]
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub enum JobResult<T> {
    Running(Job<T>),
    Complete(T),
    Dead,
}

impl<T> JobResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            Self::Complete(data) => data,
            _ => panic!("Trying to unwrap incomplete job!"),
        }
    }

    pub fn expect(self, message: &str) -> T {
        match self {
            Self::Complete(data) => data,
            _ => panic!("{}", message),
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running(_))
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_))
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead)
    }
}

#[cfg(not(feature = "parallel"))]
pub struct Job<T>(T);

#[cfg(feature = "parallel")]
pub struct Job<T>(Receiver<T>);

impl<T> Job<T> {
    pub fn new<F>(f: F) -> Job<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        #[cfg(not(feature = "parallel"))]
        {
            Job(f())
        }
        #[cfg(feature = "parallel")]
        {
            let (sender, receiver) = channel();
            rayon::spawn_fifo(move || {
                let _ = sender.send(f());
            });
            Job(receiver)
        }
    }
}

impl<T> Job<T> {
    #[cfg(not(feature = "parallel"))]
    pub fn try_consume(self) -> JobResult<T> {
        JobResult::Complete(self.0)
    }

    #[cfg(feature = "parallel")]
    pub fn try_consume(self) -> JobResult<T> {
        let receiver = self.0;
        match receiver.try_recv() {
            Ok(data) => JobResult::Complete(data),
            Err(error) => match error {
                TryRecvError::Empty => JobResult::Running(Self(receiver)),
                TryRecvError::Disconnected => JobResult::Dead,
            },
        }
    }

    #[cfg(not(feature = "parallel"))]
    pub fn consume(self) -> JobResult<T> {
        self.try_consume()
    }

    #[cfg(feature = "parallel")]
    pub fn consume(self) -> JobResult<T> {
        match self.0.recv() {
            Ok(data) => JobResult::Complete(data),
            Err(_) => JobResult::Dead,
        }
    }
}

pub struct JobsGroup<T>(Vec<JobResult<T>>);

impl<T> JobsGroup<T> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Job<T>>,
    {
        Self(iter.into_iter().map(JobResult::Running).collect())
    }

    pub fn try_consume(self) -> Result<Vec<T>, Self> {
        let result = self
            .0
            .into_iter()
            .map(|result| match result {
                JobResult::Running(job) => job.try_consume(),
                _ => result,
            })
            .collect::<Vec<_>>();
        if result.iter().all(|result| result.is_complete()) {
            Ok(result.into_iter().map(|result| result.unwrap()).collect())
        } else {
            Err(Self(result))
        }
    }

    pub fn consume(self) -> Vec<T> {
        self.0
            .into_iter()
            .filter_map(|result| match result {
                JobResult::Running(job) => match job.consume() {
                    JobResult::Complete(data) => Some(data),
                    _ => None,
                },
                JobResult::Complete(data) => Some(data),
                JobResult::Dead => None,
            })
            .collect()
    }

    pub fn is_all_complete(&self) -> bool {
        self.0.iter().all(|result| result.is_complete())
    }

    pub fn is_any_dead(&self) -> bool {
        self.0.iter().any(|result| result.is_dead())
    }

    pub fn is_any_running(&self) -> bool {
        self.0.iter().any(|result| result.is_running())
    }
}

impl<T> FromIterator<Job<T>> for JobsGroup<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Job<T>>,
    {
        Self::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_jobs() {
        fn fib(n: usize) -> (usize, Duration) {
            let timer = Instant::now();
            let mut x = (1, 1);
            for _ in 0..n {
                x = (x.1, x.0 + x.1)
            }
            (x.0, timer.elapsed())
        }

        let single = Job::new(|| fib(50));
        let group = (0..20)
            .into_iter()
            .map(|n| Job::new(move || fib(n)))
            .collect::<JobsGroup<_>>();
        println!("* Single result: {:?}", single.consume().unwrap());
        println!("* Group results: {:?}", group.consume());
    }
}
