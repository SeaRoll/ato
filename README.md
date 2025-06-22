# ATO: A Simple Task Async Runtime for `no_std` Environments

[![crates.io](https://img.shields.io/crates/v/ato.svg)](https://crates.io/crates/ato) [![docs.rs](https://docs.rs/ato/badge.svg)](https://docs.rs/ato) [![no_std](https://img.shields.io/badge/no__std-compatible-brightgreen.svg)](#)

**ATO** is a minimal asynchronous task runtime designed for `no_std` environments, making it suitable for embedded systems, operating system kernels, or other resource-constrained applications where the standard library is unavailable.

It provides a basic task spawner and a round-robin scheduler to run `Future`s to completion.

## Features

* **`no_std` Compatible:** Works in environments without the standard library (requires `alloc`).
* **Task Spawner:** Queue multiple asynchronous tasks.
* **Round-Robin Scheduling:** Tasks are polled sequentially until completion.
* **Simple Sleep Functionality:** Includes an async `sleep` function that requires a user-provided time source.
* **Fixed-Size Task Queue:** Uses `heapless::Q*` for a statically-sized task queue, configurable at compile time.
* **Simple Yield Functionality:** Allows yielding control back to the scheduler, enabling cooperative multitasking.

## Motivation

In many `no_std` contexts, a full-fledged async runtime like Tokio or async-std is too heavy or relies on operating system features that aren't available. ATO aims to provide the bare essentials for cooperative multitasking with futures in such environments.

## Installation

Add ATO to your `Cargo.toml`:

```toml
[dependencies]
ato = "1.0.4" # Replace with the desired version
```

Ensure you have an allocator set up if you're in a no_std environment, as ATO uses Box for tasks.

## Usage

Here's a basic example of how to use ATO:


```rust
use ato::{sleep, Spawner};
use core::time::Duration;

fn get_platform_time() -> Duration {
    static mut FAKE_TIME: u64 = 0;
    unsafe {
        FAKE_TIME += 10_000_000; // Increment by 10ms for example
        Duration::from_nanos(FAKE_TIME)
    }
}

fn main() {
    // Creates a spawner.
    // NOTE: The size must be power of two, e.g., 2, 4, 8, 16, etc.
    static SPAWNER: Spawner<8> = Spawner::new();

    SPAWNER
        .spawn(async {
            sleep(Duration::from_secs(1), get_platform_time).await;
        })
        .unwrap();

    // Spawn another task
    SPAWNER
        .spawn(async {
            sleep(Duration::from_millis(500), get_platform_time).await;
            SPAWNER
                .spawn(async {
                    sleep(Duration::from_secs(2), get_platform_time).await;
                })
                .unwrap();
        })
        .unwrap();

    SPAWNER.run_until_all_done().unwrap();
}
```
