use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

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
            duration,
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
/// * `time_fn`: A function pointer `fn() -> Duration` that returns the current
///   monotonic time in nanoseconds. The user must provide this.
///
/// This function is `no_std` compatible (it only uses `core` types),
/// assuming the provided `time_fn` is also `no_std` compatible.
pub fn sleep(duration: Duration, time_fn: fn() -> Duration) -> DurationSleep {
    DurationSleep::new(duration, time_fn)
}
