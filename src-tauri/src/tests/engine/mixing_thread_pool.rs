use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::time::Duration;

use super::*;

const TIMEOUT: Duration = Duration::from_secs(30);

/// Runs `body` on its own thread and fails the test if it doesn't finish in
/// time, so a deadlock in the pool shows up as a failure and not a stuck run.
fn finishes_in_time(body: impl FnOnce() + Send + 'static) {
    let (done, finished) = mpsc::channel();
    thread::spawn(move || {
        body();
        let _ = done.send(());
    });
    finished
        .recv_timeout(TIMEOUT)
        .expect("timed out: the thread pool hung");
}

fn counters(len: usize) -> Vec<AtomicUsize> {
    (0..len).map(|_| AtomicUsize::new(0)).collect()
}

fn all_ran_exactly(counters: &[AtomicUsize], times: usize) -> bool {
    counters
        .iter()
        .all(|counter| counter.load(Ordering::SeqCst) == times)
}

#[test]
fn every_task_runs_exactly_once() {
    finishes_in_time(|| {
        let pool = MixingThreadPool::new(3);
        let ran = counters(100);
        pool.run_parallel(ran.len(), &|_, index| {
            ran[index].fetch_add(1, Ordering::SeqCst);
        });
        assert!(all_ran_exactly(&ran, 1));
    });
}

#[test]
fn the_pool_is_reusable_across_many_dispatches() {
    finishes_in_time(|| {
        let pool = MixingThreadPool::new(3);
        let ran = counters(16);
        for _ in 0..1000 {
            pool.run_parallel(ran.len(), &|_, index| {
                ran[index].fetch_add(1, Ordering::SeqCst);
            });
        }
        assert!(all_ran_exactly(&ran, 1000));
    });
}

#[test]
fn dispatching_right_after_stopping_never_hangs() {
    finishes_in_time(|| {
        let pool = MixingThreadPool::new(4);
        let ran = counters(8);
        for _ in 0..3000 {
            pool.stop();
            pool.run_parallel(ran.len(), &|_, index| {
                ran[index].fetch_add(1, Ordering::SeqCst);
            });
        }
        assert!(all_ran_exactly(&ran, 3000));
    });
}

#[test]
fn stopping_from_another_thread_while_dispatching_never_hangs() {
    finishes_in_time(|| {
        let pool = Arc::new(MixingThreadPool::new(4));
        let ran = Arc::new(counters(8));

        let dispatcher = {
            let (pool, ran) = (Arc::clone(&pool), Arc::clone(&ran));
            thread::spawn(move || {
                for _ in 0..2000 {
                    pool.run_parallel(ran.len(), &|_, index| {
                        ran[index].fetch_add(1, Ordering::SeqCst);
                    });
                }
            })
        };
        let stopper = {
            let pool = Arc::clone(&pool);
            thread::spawn(move || {
                for _ in 0..2000 {
                    pool.stop();
                    thread::yield_now();
                }
            })
        };

        dispatcher.join().unwrap();
        stopper.join().unwrap();
        assert!(all_ran_exactly(&ran, 2000));
    });
}

#[test]
fn dispatching_from_two_threads_at_once_does_not_mix_them_up() {
    finishes_in_time(|| {
        let pool = Arc::new(MixingThreadPool::new(4));

        let dispatchers: Vec<_> = (0..2)
            .map(|_| {
                let pool = Arc::clone(&pool);
                thread::spawn(move || {
                    let ran = counters(8);
                    for _ in 0..1000 {
                        pool.run_parallel(ran.len(), &|_, index| {
                            ran[index].fetch_add(1, Ordering::SeqCst);
                        });
                    }
                    all_ran_exactly(&ran, 1000)
                })
            })
            .collect();

        for dispatcher in dispatchers {
            assert!(dispatcher.join().unwrap());
        }
    });
}

#[test]
fn a_task_that_panics_does_not_stop_the_others() {
    finishes_in_time(|| {
        let pool = MixingThreadPool::new(3);
        let ran = counters(20);
        pool.run_parallel(ran.len(), &|_, index| {
            if index == 3 {
                panic!("expected panic in a test task");
            }
            ran[index].fetch_add(1, Ordering::SeqCst);
        });

        for (index, counter) in ran.iter().enumerate() {
            let expected = if index == 3 { 0 } else { 1 };
            assert_eq!(counter.load(Ordering::SeqCst), expected, "task {index}");
        }
    });
}

#[test]
fn dropping_a_pool_never_hangs_whether_it_sleeps_or_is_active() {
    finishes_in_time(|| {
        for _ in 0..200 {
            drop(MixingThreadPool::new(4));

            let pool = MixingThreadPool::new(4);
            pool.start();
            pool.stop();
            drop(pool);

            let pool = MixingThreadPool::new(4);
            pool.run_parallel(4, &|_, _| {});
            drop(pool);
        }
    });
}
