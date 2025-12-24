/// A very basic multi-producer, multi-consumer channel implementation
/// using heapless's MPMC queue as the underlying storage.
/// This means that multiple producers and consumers can safely
/// send and receive items concurrently. However, this implementation
/// requires that the item type `T` implements the `Copy` trait, and
/// size must be of power of two (e.g., 2, 4, 8, 16, etc.).
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use heapless::mpmc;

/// Consumer struct for consuming items from the queue.
#[derive(Clone)]
pub struct Consumer<'a, T, const N: usize> {
    queue: &'a mpmc::MpMcQueue<T, N>,
}

impl<'a, T, const N: usize> Consumer<'a, T, N> {
    /// Receives an item from the queue.
    /// This future will wait until an item is available.
    pub async fn recv(&'a self) -> T
    where
        T: Copy,
    {
        ConsumeOnce { queue: self.queue }.await
    }

    /// Tries to receive an item from the queue.
    /// Returns `Some(item)` if successful, or `None` if the queue is empty.
    pub fn try_recv(&'a self) -> Option<T>
    where
        T: Copy,
    {
        self.queue.dequeue()
    }
}

/// Future that consumes an item once from the queue.
struct ConsumeOnce<'a, T, const N: usize> {
    queue: &'a mpmc::MpMcQueue<T, N>,
}

impl<'a, T, const N: usize> Future for ConsumeOnce<'a, T, N>
where
    T: Copy,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.queue.dequeue() {
            Some(value) => Poll::Ready(value),
            None => Poll::Pending,
        }
    }
}

/// Producer struct for producing items into the queue.
#[derive(Clone)]
pub struct Producer<'a, T, const N: usize> {
    queue: &'a mpmc::MpMcQueue<T, N>,
}

impl<'a, T, const N: usize> Producer<'a, T, N> {
    /// Sends an item into the queue.
    /// This future will wait until there is space in the queue.
    pub async fn send(&'a self, item: T)
    where
        T: Copy,
    {
        ProduceOnce {
            queue: self.queue,
            item,
        }
        .await
    }

    /// Tries to send an item into the queue.
    /// Returns `Ok(())` if successful, or `Err(item)` if the queue is full.
    pub fn try_send(&'a self, item: T) -> Result<(), T>
    where
        T: Copy,
    {
        self.queue.enqueue(item)
    }
}

/// Future that produces an item once into the queue.
struct ProduceOnce<'a, T, const N: usize> {
    queue: &'a mpmc::MpMcQueue<T, N>,
    item: T,
}

impl<'a, T, const N: usize> Future for ProduceOnce<'a, T, N>
where
    T: Copy,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.queue.enqueue(self.item) {
            Ok(()) => Poll::Ready(()),
            Err(_) => Poll::Pending,
        }
    }
}

/// Creates a new multi-producer, multi-consumer channel.
pub fn new_channel<'a, T, const N: usize>(
    queue: &'a mpmc::MpMcQueue<T, N>,
) -> (Producer<'a, T, N>, Consumer<'a, T, N>) {
    (Producer { queue }, Consumer { queue })
}

/// Channel macro creates a multi-producer, multi-consumer channel with the specified type and capacity.
#[macro_export]
macro_rules! channel {
    // Usage: task!(task_variable_name, { async_code... });
    ($name:ident, $type:ty, $size:expr) => {
        let q = heapless::mpmc::MpMcQueue::<$type, $size>::new();

        // 3. Create the handle with the user-provided name.
        // We assume TaskHandle::new is accessible here.
        let $name = $crate::channels::new_channel(&q);
    };
}
