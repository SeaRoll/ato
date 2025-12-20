use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A future that yields execution back to the scheduler once.
/// This allows a task to voluntarily pause and let other tasks run.
#[derive(Debug, Default)]
pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            // We have already yielded once, so now we can complete.
            Poll::Ready(())
        } else {
            // We have not yielded yet. Set the flag and return Pending.
            // The executor will re-queue the task and poll it again later.
            self.yielded = true;
            Poll::Pending
        }
    }
}

/// Creates a future that will yield execution back to the scheduler once.
///
/// This allows a task to voluntarily pause and let other tasks run.
pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}
