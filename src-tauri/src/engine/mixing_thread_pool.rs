use std::{
    any::Any,
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use log::info;
use parking_lot::{Condvar, Mutex};

type TaskFn = fn(*const (), usize, usize);

pub fn max_worker_count() -> usize {
    let cores = num_cpus::get_physical();
    cores.saturating_sub(2).max(1)
}

pub struct MixingThreadPool {
    /// Held while dispatching or changing whether the pool is active, so those
    /// can never overlap: two dispatches would trample the shared task state, and
    /// a stop landing mid-dispatch would put workers to sleep with work pending.
    dispatch_lock: Mutex<()>,
    workers: Vec<Worker>,
    shared: Arc<SharedState>,
}

impl MixingThreadPool {
    pub fn new(worker_count: usize) -> Self {
        let worker_count = worker_count.max(1);
        let shared = Arc::new(SharedState {
            active: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            render_generation: AtomicU64::new(0),
            completed_generation: AtomicU64::new(0),
            next_index: AtomicUsize::new(0),
            task_count: AtomicUsize::new(0),
            remaining_workers: AtomicUsize::new(0),
            task_ptr: AtomicPtr::new(ptr::null_mut()),
            task_fn: AtomicPtr::new(ptr::null_mut()),
            sleep_lock: Mutex::new(()),
            work_cv: Condvar::new(),
        });

        let mut workers = Vec::with_capacity(worker_count);

        for worker_id in 0..worker_count {
            let shared = Arc::clone(&shared);
            let handle = thread::spawn(move || worker_loop(worker_id, shared));

            workers.push(Worker {
                handle: Some(handle),
            });
        }

        Self {
            dispatch_lock: Mutex::new(()),
            workers,
            shared,
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn start(&self) {
        let _dispatch = self.dispatch_lock.lock();
        self.activate();
    }

    /// Lets the workers go to sleep. Waits for a dispatch in progress on
    /// another thread to finish first.
    pub fn stop(&self) {
        let _dispatch = self.dispatch_lock.lock();
        self.wait();
        self.set_active(false);
    }

    /// Runs `task(worker_id, task_index)` for every index in `0..task_count`
    /// across the workers, and returns once all of them are done. Calls from
    /// several threads take turns.
    pub fn run_parallel<F>(&self, task_count: usize, task: &F)
    where
        F: Fn(usize, usize) + Sync,
    {
        if task_count == 0 {
            return;
        }

        let _dispatch = self.dispatch_lock.lock();
        self.wait();

        self.shared
            .task_fn
            .store(call_task::<F> as *mut (), Ordering::Release);
        self.shared
            .task_ptr
            .store(task as *const F as *mut (), Ordering::Release);
        self.shared.next_index.store(0, Ordering::Release);
        self.shared.task_count.store(task_count, Ordering::Release);
        self.shared
            .remaining_workers
            .store(self.worker_count(), Ordering::Release);

        self.activate();
        let generation = self.shared.render_generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.wait_for_generation(generation);

        self.shared
            .task_ptr
            .store(ptr::null_mut(), Ordering::Release);
        self.shared
            .task_fn
            .store(ptr::null_mut(), Ordering::Release);
    }

    pub fn wait(&self) {
        let generation = self.shared.render_generation.load(Ordering::Acquire);
        self.wait_for_generation(generation);
    }

    pub fn shutdown(&mut self) {
        {
            // Changed under the lock a sleeping worker checks these under, so a
            // worker can't check them, miss this notification, and sleep on.
            let _sleep = self.shared.sleep_lock.lock();
            self.shared.shutdown.store(true, Ordering::Release);
            self.shared.active.store(false, Ordering::Release);
            self.shared.work_cv.notify_all();
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }

    /// Wakes the workers if they were asleep. Callers hold `dispatch_lock`, so
    /// `active` only ever changes one thread at a time.
    fn activate(&self) {
        if !self.shared.active.load(Ordering::Acquire) {
            self.set_active(true);
        }
    }

    fn set_active(&self, active: bool) {
        // Changed under the lock a sleeping worker checks it under, so a worker
        // can't check it, miss this notification, and sleep on.
        let _sleep = self.shared.sleep_lock.lock();
        self.shared.active.store(active, Ordering::Release);
        self.shared.work_cv.notify_all();
    }
    fn wait_for_generation(&self, generation: u64) {
        while self.shared.completed_generation.load(Ordering::Acquire) < generation
            && !self.shared.shutdown.load(Ordering::Acquire)
        {
            std::hint::spin_loop();
        }
    }
}

impl Drop for MixingThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Worker {
    handle: Option<JoinHandle<()>>,
}

struct SharedState {
    active: AtomicBool,
    shutdown: AtomicBool,
    render_generation: AtomicU64,
    completed_generation: AtomicU64,
    next_index: AtomicUsize,
    task_count: AtomicUsize,
    remaining_workers: AtomicUsize,
    task_ptr: AtomicPtr<()>,
    task_fn: AtomicPtr<()>,
    sleep_lock: Mutex<()>,
    work_cv: Condvar,
}

fn worker_loop(worker_id: usize, shared: Arc<SharedState>) {
    let mut seen_generation = 0;

    loop {
        if shared.shutdown.load(Ordering::Acquire) {
            return;
        }

        if !shared.active.load(Ordering::Acquire) {
            let mut guard = shared.sleep_lock.lock();
            while !shared.shutdown.load(Ordering::Acquire) && !shared.active.load(Ordering::Acquire)
            {
                shared.work_cv.wait(&mut guard);
            }
            continue;
        }

        let generation = shared.render_generation.load(Ordering::Acquire);
        if generation == seen_generation {
            std::hint::spin_loop();
            continue;
        }
        seen_generation = generation;

        run_worker_batch(worker_id, generation, &shared);
    }
}

fn run_worker_batch(worker_id: usize, generation: u64, shared: &SharedState) {
    let task_count = shared.task_count.load(Ordering::Acquire);
    let task_ptr = shared.task_ptr.load(Ordering::Acquire) as *const ();
    let task_fn = shared.task_fn.load(Ordering::Acquire);

    while let Some(task_index) = claim_next_index(shared, task_count) {
        if !task_fn.is_null() {
            if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let task_fn: TaskFn = unsafe { std::mem::transmute(task_fn) };
                task_fn(task_ptr, worker_id, task_index);
            })) {
                log_worker_panic(worker_id, payload);
            }
        }
    }

    if shared.remaining_workers.fetch_sub(1, Ordering::AcqRel) == 1 {
        shared
            .completed_generation
            .store(generation, Ordering::Release);
    }
}

fn claim_next_index(shared: &SharedState, task_count: usize) -> Option<usize> {
    let task_index = shared.next_index.fetch_add(1, Ordering::Relaxed);
    (task_index < task_count).then_some(task_index)
}

fn call_task<F>(task_ptr: *const (), worker_id: usize, task_index: usize)
where
    F: Fn(usize, usize),
{
    let task = unsafe { &*(task_ptr as *const F) };
    task(worker_id, task_index);
}

fn log_worker_panic(worker_id: usize, payload: Box<dyn Any + Send>) {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic payload");

    info!("MixingThreadPool worker {worker_id} panicked: {message}");
}

#[cfg(test)]
#[path = "../tests/engine/mixing_thread_pool.rs"]
mod tests;
