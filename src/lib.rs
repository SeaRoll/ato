//! ATO: A simple task async runtime for no_std environments with no alloc required.
//!
//! This library provides a basic task spawner and runner, allowing you to spawn
//! futures and run them to completion in a queued manner (FIFO).
//!
//! It is designed to be used in environments without the standard library (`no_std`) or no heap
//! allocation support, making it suitable for embedded systems or other constrained environments.
//!
//! # Features
//! - Task spawner that can queue multiple futures.
//! - FIFO scheduling of tasks.
//! - Simple sleep functionality using `core::time::Duration`.
//! - Simple yielding functionality to allow tasks to yield control back to the scheduler.
//!
//! Examples can be found in the examples folder, demonstrating how to use the `Spawner` and `sleep` functionality.

#![no_std]

mod sleep;
mod yield_now;

pub use crate::sleep::sleep;
pub use crate::yield_now::yield_now;

use core::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

// --- ERROR & WAKER ---
#[derive(Debug)]
pub enum Error {
    TaskQueueFail,
}

unsafe fn nop(_: *const ()) {}
unsafe fn nop_clone(_data: *const ()) -> RawWaker {
    RawWaker::new(ptr::null(), &VTABLE)
}
static VTABLE: RawWakerVTable = RawWakerVTable::new(nop_clone, nop, nop, nop);

// Task type alias
type Task<'a> = Pin<&'a mut (dyn Future<Output = ()> + Send + Sync)>;

/// A simple task spawner and runner for `no_std` environments.
/// The `Spawner` can queue and run multiple tasks (futures) in a FIFO manner.
/// # Type Parameters
/// - `N`: The maximum number of tasks that can be queued. Must be a power of two (e.g., 2, 4, 8, 16, etc.).
pub struct Spawner<'a, const N: usize> {
    tasks: heapless::mpmc::MpMcQueue<Task<'a>, N>,
    waker: Waker,
}

impl<'a, const N: usize> Default for Spawner<'a, N> {
    fn default() -> Self {
        let raw_waker = RawWaker::new(ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        Spawner {
            tasks: heapless::mpmc::MpMcQueue::new(),
            waker,
        }
    }
}

impl<'a, const N: usize> Spawner<'a, N> {
    /// Spawns a task. Make sure to use the `task!` macro to pin the future to the stack.
    pub fn spawn(
        &self,
        future: &'a mut (dyn Future<Output = ()> + Send + Sync),
    ) -> Result<(), Error> {
        // We re-pin the reference. This is safe because the reference we receive
        // is already mutable and valid for 'a.
        let pinned_task = unsafe { Pin::new_unchecked(future) };

        match self.tasks.enqueue(pinned_task) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::TaskQueueFail),
        }
    }

    /// Runs tasks until all are completed.
    pub fn run_until_all_done(&self) -> Result<(), Error> {
        let mut cx = Context::from_waker(&self.waker);

        while let Some(mut task) = self.tasks.dequeue() {
            match task.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    if self.tasks.enqueue(task).is_err() {
                        return Err(Error::TaskQueueFail);
                    }
                }
            }
        }
        Ok(())
    }
}

/// task! macro is used to create a pinned async block.
/// It pins the async block to the stack, making it suitable for spawning
/// in the ATO runtime.
#[macro_export]
macro_rules! task {
    ( $($body:tt)* ) => {
        core::pin::pin!(async move { $($body)* })
    };
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::time::Duration;
    use heapless::mpmc::Q2;
    use std::{sync::Arc, sync::Mutex, time::Instant, vec::Vec};

    use super::*;

    // --- Time source for `std` test environments ---
    // We need a static `Instant` to serve as our epoch for calculating monotonic time.
    // `std::sync::OnceLock` initializes this safely for concurrent tests (though these are single-threaded).
    // This is specifically for the test environment.
    static TEST_EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

    /// Initializes (if not already) and returns the test's epoch Instant.
    fn get_test_epoch() -> Instant {
        *TEST_EPOCH.get_or_init(Instant::now)
    }

    /// Returns the current monotonic time as a Duration since the test epoch.
    /// This function is suitable for passing as `time_fn` to `sleep` in a `std` test environment.
    /// It's a non-capturing function, so it can be cast to a `fn` pointer.
    fn get_current_test_time_duration() -> Duration {
        let epoch = get_test_epoch(); // Ensure epoch is initialized
        Instant::now().duration_since(epoch)
    }

    async fn hello() {
        std::println!("Hello, world!");
    }

    #[test]
    fn test_spawner_sleep() {
        let spawner: Spawner<8> = Spawner::default();

        // Initialize the epoch at the start of tests that use it.
        // This ensures a consistent time base for each test run if tests run sequentially
        // or if the OnceLock hasn't been initialized yet.
        let _ = get_test_epoch();

        let sleep_duration = Duration::from_millis(10);

        let mut pinned_future = task!({
            sleep(sleep_duration, get_current_test_time_duration).await;
            hello().await;
        });

        if let Err(_) = spawner.spawn(&mut pinned_future) {
            panic!("Failed to spawn task");
        }

        if let Err(_) = spawner.run_until_all_done() {
            panic!("Failed to run tasks");
        }
    }

    #[test]
    fn test_spawner_queues() {
        static Q: Q2<u8> = Q2::new();
        let spawner: Spawner<2> = Spawner::default();
        let _ = get_test_epoch();

        let mut dequeue_future = task!({
            loop {
                sleep(Duration::from_millis(10), get_current_test_time_duration).await;
                if let Some(_) = Q.dequeue() {
                    break;
                }
            }
        });

        spawner
            .spawn(&mut dequeue_future)
            .expect("Failed to spawn task");

        let mut enqueue_future = task!({
            sleep(Duration::from_secs(1), get_current_test_time_duration).await;
            Q.enqueue(42).unwrap();
        });
        spawner
            .spawn(&mut enqueue_future)
            .expect("Failed to spawn task");

        spawner.run_until_all_done().expect("Failed to run tasks");
    }

    #[test]
    fn test_yield_now() {
        let spawner: Spawner<8> = Spawner::default();
        let lock = Arc::new(Mutex::new(Vec::new()));

        let lock_clone = lock.clone();
        let mut first_future = task!({
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(1);
            }
            yield_now().await; // Yield control back to the scheduler
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(3);
            }
        });

        let lock_clone = lock.clone();
        let mut second_future = task!({
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(2);
            }
        });

        spawner
            .spawn(&mut first_future)
            .expect("Failed to spawn first future");
        spawner
            .spawn(&mut second_future)
            .expect("Failed to spawn second future");
        spawner.run_until_all_done().unwrap();

        // check that the lock was accessed correctly
        let num = lock.lock().unwrap();
        assert_eq!(
            *num,
            Vec::from([1, 2, 3]),
            "Lock was not accessed correctly"
        );
    }
}
