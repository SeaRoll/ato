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

pub mod channels;
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

/// A type alias for a pinned future that outputs `()`.
type Task<'a> = Pin<&'a mut (dyn Future<Output = ()> + Send + Sync)>;

/// A handle to a task (future) that can be spawned in the ATO runtime.
pub struct TaskHandle<'a> {
    inner: Task<'a>,
}

impl<'a> TaskHandle<'a> {
    /// Creates a new TaskHandle from a future reference.
    pub fn new(future: Pin<&'a mut (dyn Future<Output = ()> + Send + Sync)>) -> Self {
        TaskHandle { inner: future }
    }
}

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
    pub fn spawn(&self, future: TaskHandle<'a>) -> Result<(), Error> {
        match self.tasks.enqueue(future.inner) {
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

    /// Runs a single task if available.
    /// Could be useful in environments where you want to interleave task execution
    /// with other processing.
    pub fn run_once(&self) -> Result<(), Error> {
        let mut cx = Context::from_waker(&self.waker);

        if let Some(mut task) = self.tasks.dequeue() {
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

/// Macro to simplify spawning tasks with the `Spawner`.
#[macro_export]
macro_rules! spawn_task {
    // Usage: task!(spawner, result, { async_code... });
    ($spawner:expr, $result:ident, $body:expr) => {
        // 1. Create the future variable in the CURRENT scope
        let future = async move { $body };

        // 2. Pin it. `core::pin::pin!` creates a `Pin<&mut T>`
        // that borrows the local `future` variable we just created.
        let pinned_fut = core::pin::pin!(future);

        // 3. Create the handle.
        let task_handle = $crate::TaskHandle::new(pinned_fut);

        // 4. Spawn the task using the provided spawner.
        let $result = $spawner.spawn(task_handle);
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

        spawn_task!(spawner, res, {
            sleep(sleep_duration, get_current_test_time_duration).await;
            hello().await;
        });
        res.expect("Failed to spawn task");

        if let Err(_) = spawner.run_until_all_done() {
            panic!("Failed to run tasks");
        }
    }

    #[test]
    fn test_spawner_queues() {
        static Q: Q2<u8> = Q2::new();
        let spawner: Spawner<2> = Spawner::default();
        let _ = get_test_epoch();

        spawn_task!(spawner, res, {
            loop {
                sleep(Duration::from_millis(10), get_current_test_time_duration).await;
                if let Some(_) = Q.dequeue() {
                    break;
                }
            }
        });
        res.expect("Failed to spawn task");

        spawn_task!(spawner, res, {
            sleep(Duration::from_secs(1), get_current_test_time_duration).await;
            Q.enqueue(42).unwrap();
        });
        res.expect("Failed to spawn task");

        spawner.run_until_all_done().expect("Failed to run tasks");
    }

    #[test]
    fn test_yield_now() {
        let spawner: Spawner<8> = Spawner::default();
        let lock = Arc::new(Mutex::new(Vec::new()));

        let lock_clone = lock.clone();
        spawn_task!(spawner, res, {
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
        res.expect("Failed to spawn task");

        let lock_clone = lock.clone();
        spawn_task!(spawner, res, {
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(2);
            }
        });
        res.expect("Failed to spawn task");
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
