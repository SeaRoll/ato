# ATO: A Simple Task Async Runtime for `no_std` Environments

[![crates.io](https://img.shields.io/crates/v/ato.svg)](https://crates.io/crates/ato) [![docs.rs](https://docs.rs/ato/badge.svg)](https://docs.rs/ato) [![no_std](https://img.shields.io/badge/no__std-compatible-brightgreen.svg)](#)

**ATO** is a minimal asynchronous task runtime designed for `no_std` environments, making it suitable for embedded systems, operating system kernels, or other resource-constrained applications where the standard library is unavailable.

It provides a basic task spawner and a round-robin scheduler to run `Future`s to completion.

## Features

* **`no_std` Compatible:** Works in environments without the standard library (requires `alloc`).
* **Task Spawner:** Queue multiple asynchronous tasks.
* **Round-Robin Scheduling:** Tasks are polled sequentially until completion.
* **Simple Sleep Functionality:** Includes an async `sleep` function that requires a user-provided time source.
* **Fixed-Size Task Queue:** Uses `heapless::Deque` for a statically-sized task queue, configurable at compile time.

## Motivation

In many `no_std` contexts, a full-fledged async runtime like Tokio or async-std is too heavy or relies on operating system features that aren't available. ATO aims to provide the bare essentials for cooperative multitasking with futures in such environments.

## Installation

Add ATO to your `Cargo.toml`:

```toml
[dependencies]
ato = "1.0.1" # Replace with the desired version
```

Ensure you have an allocator set up if you're in a no_std environment, as ATO uses Box for tasks.

## Usage

Here's a basic example of how to use ATO:


```rust
#![no_std]
extern crate alloc;

use ato::{Spawner, sleep, Error};
use core::time::Duration;

// --- You need to provide a time source function ---
// This function must return the current monotonic time as a Duration.
// The exact implementation will depend on your hardware/platform.
// For example, it might read a hardware timer.
fn get_platform_time() -> Duration {
    // Replace this with your actual time-keeping logic
    // For demonstration, let's assume a dummy incrementing counter in nanoseconds.
    // In a real scenario, this would interface with a hardware timer.
    static mut FAKE_TIME: u64 = 0;
    unsafe {
        FAKE_TIME += 10_000_000; // Increment by 10ms for example
        Duration::from_nanos(FAKE_TIME)
    }
}
// --- End of platform-specific time source ---

fn main() -> Result<(), Error> {
    // Create a new spawner with a default capacity for 8 tasks.
    // You can specify a different capacity, e.g., Spawner::<16>::new().
    let mut spawner: Spawner<8> = Spawner::new();

    // Spawn a task that sleeps for 1 second
    spawner.spawn(async {
        // The first argument is the duration to sleep.
        // The second argument is a function pointer to your time source.
        sleep(Duration::from_secs(1), get_platform_time).await;
        // In a real application, you might print to a console or toggle an LED.
        // For no_std, printing requires a platform-specific implementation.
        // println!("Task 1: Slept for 1 second!");
    })?;

    // Spawn another task
    spawner.spawn(async {
        sleep(Duration::from_millis(500), get_platform_time).await;
        // println!("Task 2: Slept for 500 milliseconds!");
        sleep(Duration::from_millis(500), get_platform_time).await;
        // println!("Task 2: Slept for another 500 milliseconds!");
    })?;

    // Run all tasks until they are done
    // This will block and poll tasks in a round-robin fashion.
    spawner.run_until_all_done()?;

    // println!("All tasks completed!");
    Ok(())
}
```
