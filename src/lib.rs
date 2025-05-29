//! ATO: A simple task async runtime for no_std environments.
//!
//! This library provides a basic task spawner and runner, allowing you to spawn
//! futures and run them to completion in a round-robin fashion.
//!
//! It is designed to be used in environments without the standard library (`no_std`),
//! making it suitable for embedded systems or other constrained environments.
//!
//! # Features
//! - Task spawner that can queue multiple futures.
//! - Round-robin scheduling of tasks.
//! - Simple sleep functionality using `core::time::Duration`.
//!
//! # Example
//! ```
//! use ato::{Spawner, sleep};
//! use core::time::Duration;
//!
//! // Create a new spawner
//! let mut spawner<8> = Spawner::new();
//!
//! // Spawn a task that sleeps for 100 milliseconds
//! spawner.spawn(async {
//!    sleep(Duration::from_secs(1), get_current_test_time_duration).await;
//! }).unwrap();
//!
//! // Run all tasks until they are done
//! spawner.run_until_all_done().unwrap();
//! ```

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::time::Duration;

use core::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

#[derive(Debug)]
pub enum Error {
    TaskQueueFail,
}

// These functions and VTABLE are used to create a simple Waker
// that doesn't actually do any waking. It's suitable for a
// busy-looping executor like block_on or this simple Spawner.
unsafe fn nop(_: *const ()) {}

unsafe fn nop_clone(_data: *const ()) -> RawWaker {
    // When cloning the waker, we effectively create a new one with the same null data and vtable.
    RawWaker::new(ptr::null(), &VTABLE)
}

// The VTABLE for our simple Waker.
// The functions are: clone, wake, wake_by_ref, drop.
static VTABLE: RawWakerVTable = RawWakerVTable::new(nop_clone, nop, nop, nop);

// --- New Spawner Code ---

/// A type alias for a task. A task is a pinned, boxed future that outputs `()`.
/// The `'static` lifetime means the future cannot hold any non-static references.
type Task = Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;

/// A simple task spawner and runner.
/// It can queue multiple futures (tasks) and run them to completion.
pub struct Spawner<const N: usize, Q = heapless::mpmc::MpMcQueue<Task, N>> {
    /// A queue of tasks to be run.
    tasks: Q,
    /// A reusable waker for polling tasks.
    waker: Waker,
}

impl<const N: usize> Spawner<N> {
    /// Creates a new, empty `Spawner`.
    pub const fn new() -> Self {
        // Create a raw waker using the same VTABLE as block_on.
        // This waker doesn't do anything special, which is fine for a simple,
        // single-threaded, busy-polling spawner.
        let raw_waker = RawWaker::new(ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        Spawner {
            tasks: heapless::mpmc::MpMcQueue::new(),
            waker,
        }
    }

    /// Spawns a new task.
    /// The provided future will be boxed, pinned, and added to the task queue.
    ///
    /// # Arguments
    /// * `future`: The future to spawn. It must be `'static` and output `()`.
    pub fn spawn<F>(&self, future: F) -> Result<(), Error>
    where
        F: Future<Output = ()> + 'static + Send + Sync,
    {
        match self.tasks.enqueue(Box::pin(future)) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::TaskQueueFail),
        }
    }

    /// Runs all spawned tasks to completion.
    /// This method will block and continuously poll tasks in a round-robin fashion
    /// until all tasks have completed.
    ///
    /// Note: If any task never completes (e.g., an infinite loop that never yields `Poll::Ready`),
    /// this method will also loop forever.
    pub fn run_until_all_done(&self) -> Result<(), Error> {
        let mut cx = Context::from_waker(&self.waker);

        // Loop as long as there are tasks in the queue.
        while let Some(mut task) = self.tasks.dequeue() {
            match task.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {
                    // Task is complete. It's already removed from the queue, so we do nothing.
                }
                Poll::Pending => {
                    // println!("Task is pending, re-queuing...");
                    // Task is not yet complete. Push it to the back of the queue
                    // to be polled again later. This implements a simple round-robin scheduling.
                    if let Err(_) = self.tasks.enqueue(task) {
                        return Err(Error::TaskQueueFail);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DurationSleep {
    start_time: Option<Duration>,
    duration: Duration,
    time_fn: fn() -> Duration,
}

impl DurationSleep {
    /// Creates a new `DurationSleep` future.
    ///
    /// # Arguments
    /// * `duration`: The minimum duration for which to sleep.
    /// * `time_fn`: A function that returns the current monotonic time in nanoseconds.
    pub fn new(duration: Duration, time_fn: fn() -> Duration) -> Self {
        DurationSleep {
            start_time: None,
            duration: duration,
            time_fn,
        }
    }
}

impl Future for DurationSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_time = (self.time_fn)();

        let start_time = match self.start_time {
            Some(time) => time,
            None => {
                self.as_mut().start_time = Some(current_time);
                current_time
            }
        };
        if self.duration == Duration::ZERO {
            return Poll::Ready(());
        }

        let elapsed = current_time.saturating_sub(start_time);

        if elapsed >= self.duration {
            // The sleep duration has passed.
            Poll::Ready(())
        } else {
            // Still sleeping, return Pending.
            Poll::Pending
        }
    }
}

/// Creates a future that will "sleep" for the specified `Duration`.
///
/// # Arguments
/// * `duration`: The `core::time::Duration` to sleep for.
/// * `time_fn`: A function pointer `fn() -> u64` that returns the current
///              monotonic time in nanoseconds. The user must provide this.
///
/// This function is `no_std` compatible (it only uses `core` types),
/// assuming the provided `time_fn` is also `no_std` compatible.
pub fn sleep(duration: Duration, time_fn: fn() -> Duration) -> DurationSleep {
    DurationSleep::new(duration, time_fn)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::time::Instant;

    use heapless::mpmc::Q2;

    use super::*;

    static SPAWNER: Spawner<4> = Spawner::new();
    static Q: Q2<u8> = Q2::new();

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

    #[test]
    fn test_spawner_sleep() {
        // Initialize the epoch at the start of tests that use it.
        // This ensures a consistent time base for each test run if tests run sequentially
        // or if the OnceLock hasn't been initialized yet.
        let _ = get_test_epoch();

        let sleep_duration = Duration::from_millis(10);

        if let Err(_) = SPAWNER.spawn(async move {
            sleep(sleep_duration, get_current_test_time_duration).await;
        }) {
            panic!("Failed to spawn task");
        }

        if let Err(_) = SPAWNER.run_until_all_done() {
            panic!("Failed to run tasks");
        }
    }

    #[test]
    fn test_spawner_queues() {
        let _ = get_test_epoch();

        if let Err(_) = SPAWNER.spawn(async move {
            loop {
                sleep(Duration::from_millis(10), get_current_test_time_duration).await;
                if let Some(_) = Q.dequeue() {
                    break;
                }
            }
        }) {
            panic!("Failed to spawn task");
        }

        if let Err(_) = SPAWNER.spawn(async move {
            sleep(Duration::from_secs(1), get_current_test_time_duration).await;
            Q.enqueue(42).unwrap();

            // Create another task that will dequeue
            SPAWNER
                .spawn(async move {
                    sleep(Duration::from_secs(1), get_current_test_time_duration).await;
                    Q.enqueue(42).unwrap();
                })
                .unwrap();
        }) {
            panic!("Failed to spawn task");
        }

        if let Err(e) = SPAWNER.run_until_all_done() {
            panic!("Failed to run tasks - {:?}", e);
        }
    }
}
