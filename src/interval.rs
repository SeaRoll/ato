use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// A structure that ticks at regular intervals.
#[derive(Debug)]
pub struct Interval {
    next_tick: Duration,
    period: Duration,
    time_fn: fn() -> Duration,
}

impl Interval {
    /// Creates a new `Interval` that ticks at the specified `period`.
    ///
    /// The first tick completes immediately (burst strategy), similar to Tokio.
    pub fn new(period: Duration, time_fn: fn() -> Duration) -> Self {
        let now = time_fn();
        Self {
            next_tick: now,
            period,
            time_fn,
        }
    }

    /// Creates a future that completes when the next interval is reached.
    ///
    /// The returned future borrows the interval, allowing the `Interval` to
    /// track state across multiple calls.
    pub fn tick(&mut self) -> IntervalTick<'_> {
        IntervalTick { interval: self }
    }

    /// Resets the interval to start ticking from the current instant.
    ///
    /// This is useful if the system was paused or lagged significantly and
    /// you want to skip the "burst" of missed ticks.
    pub fn reset(&mut self) {
        self.next_tick = (self.time_fn)();
    }
}

/// A future returned by `Interval::tick`.
pub struct IntervalTick<'a> {
    interval: &'a mut Interval,
}

impl Future for IntervalTick<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let now = (this.interval.time_fn)();

        if now >= this.interval.next_tick {
            this.interval.next_tick += this.interval.period;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
