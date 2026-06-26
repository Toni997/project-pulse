use std::{
    any::Any,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

fn max_worker_count() -> usize {
    let cores = num_cpus::get_physical();
    cores.saturating_sub(2).max(1)
}

pub struct WorkerPool {
    workers: Vec<Worker>,
    shared: Arc<(Mutex<SharedState>, Condvar, Condvar)>,
}

struct Worker {
    handle: Option<JoinHandle<()>>,
}

struct SharedState {
    job: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    remaining: usize,
    generation: u64,
    shutdown: bool,
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> Self {
        let worker_count = worker_count.max(1);
        let shared = Arc::new((
            Mutex::new(SharedState {
                job: None,
                remaining: 0,
                generation: 0,
                shutdown: false,
            }),
            Condvar::new(),
            Condvar::new(),
        ));

        let mut workers = Vec::with_capacity(worker_count);

        for worker_id in 0..worker_count {
            let shared = Arc::clone(&shared);

            let handle = thread::spawn(move || {
                let (state_lock, work_cv, done_cv) = &*shared;
                let mut last_seen_generation = 0;

                loop {
                    let (job, generation) = {
                        let mut state = state_lock.lock().unwrap();
                        while !state.shutdown && state.generation == last_seen_generation {
                            state = work_cv.wait(state).unwrap();
                        }

                        if state.shutdown {
                            return;
                        }

                        last_seen_generation = state.generation;
                        (state.job.clone(), state.generation)
                    };

                    if let Some(job) = job {
                        if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            job(worker_id);
                        })) {
                            log_worker_panic(worker_id, payload);
                        }

                        let mut state = state_lock.lock().unwrap();
                        if state.generation == generation && state.remaining > 0 {
                            state.remaining -= 1;
                            if state.remaining == 0 {
                                done_cv.notify_one();
                            }
                        }
                    }
                }
            });

            workers.push(Worker {
                handle: Some(handle),
            });
        }

        Self {
            workers,
            shared,
        }
    }

    pub fn run_blocking<F>(&self, f: F)
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        let worker_count = self.workers.len();
        let (state_lock, work_cv, done_cv) = &*self.shared;

        let mut state = state_lock.lock().unwrap();
        while state.job.is_some() && !state.shutdown {
            state = done_cv.wait(state).unwrap();
        }
        if state.shutdown {
            return;
        }

        state.remaining = worker_count;
        state.job = Some(Arc::new(f));
        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        work_cv.notify_all();

        while state.remaining != 0 && state.generation == generation && !state.shutdown {
            state = done_cv.wait(state).unwrap();
        }
        if state.generation == generation {
            state.job = None;
            done_cv.notify_all();
        }
    }

    pub fn shutdown(&mut self) {
        let (state_lock, work_cv, done_cv) = &*self.shared;
        {
            let mut state = state_lock.lock().unwrap();
            state.shutdown = true;
            state.job = None;
            work_cv.notify_all();
            done_cv.notify_all();
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn log_worker_panic(worker_id: usize, payload: Box<dyn Any + Send>) {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic payload");

    eprintln!("WorkerPool worker {worker_id} panicked: {message}");
}
