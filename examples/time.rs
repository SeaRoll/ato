use ato::{sleep, Spawner};
use core::time::Duration;
use std::time::Instant;

const SPAWNER_SIZE: usize = 2; // Must be a power of two, e.g., 2, 4, 8, 16, etc.
static SPAWNER: Spawner<SPAWNER_SIZE> = Spawner::new();
static TEST_EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_platform_time() -> Duration {
    let epoch = TEST_EPOCH.get_or_init(Instant::now);
    Instant::now().duration_since(*epoch)
}

fn main() {
    SPAWNER
        .spawn(async {
            let start = Instant::now();
            sleep(Duration::from_millis(100), get_platform_time).await;
            let elapsed = Instant::now().duration_since(start);
            println!(
                "Task 1 completed after {:?} milliseconds",
                elapsed.as_millis()
            );
        })
        .unwrap();

    SPAWNER.run_until_all_done().unwrap();
}
