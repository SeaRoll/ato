# ATO: A Simple Task Async Runtime for `no_std` and `no_alloc` Environments

[![crates.io](https://img.shields.io/crates/v/ato.svg)](https://crates.io/crates/ato) [![docs.rs](https://docs.rs/ato/badge.svg)](https://docs.rs/ato) [![no_std](https://img.shields.io/badge/no__std-compatible-brightgreen.svg)](#)

**ATO** is a minimal asynchronous task runtime designed for `no_std` and `no_alloc` environments, making it suitable for embedded systems, operating system kernels, or other resource-constrained applications where the standard library is unavailable.

It provides a basic task spawner and a round-robin scheduler to run `Future`s to completion.

## Features

* **`no_std` Compatible:** Works in environments without the standard library.
* **No Allocator Needed:** Designed to operate without dynamic memory allocation.
* **Task Spawner:** Queue multiple asynchronous tasks.
* **Round-Robin Scheduling:** Tasks are polled sequentially until completion.
* **Simple Sleep Functionality:** Includes an async `sleep` function that requires a user-provided time source.
* **Fixed-Size Task Queue:** Uses `heapless::Q*` for a statically-sized task queue, configurable at compile time.
* **Simple Yield Functionality:** Allows yielding control back to the scheduler, enabling cooperative multitasking.

## Motivation

In many `no_std` contexts, a full-fledged async runtime like Tokio or async-std is too heavy or relies on 
operating system features that aren't available. ATO aims to provide the bare essentials for cooperative 
multitasking with futures in such environments.

## Migration from V1 to V2
The v2 release of ATO introduces breaking changes due to usage of no allocator required. This means that the `Spawner`
is now not `'static` but rather scoped to the stack frame it is created in. This allows ATO to work in `no_alloc` environments.
Please take a look at the example for how to create and use a `Spawner` in v2.

## Installation

Add ATO to your `Cargo.toml`:

```toml
[dependencies]
ato = "2.0.0" # Replace with the desired version
```

## Usage

Here's a basic example of how to use ATO:

```rust
use ato::Spawner;

const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.

fn main() {
    // create a spawner with the specified size
    let spawner: Spawner<SPAWNER_SIZE> = Spawner::new();

    // create a simple task that prints a message
    let mut task = ato::task!({
        println!("Task 1 started");
    });
    spawner.spawn(&mut task).unwrap();

    // run until all tasks are done running
    spawner.run_until_all_done().unwrap();
}
```
