use core::time::Duration;
use std::time::Instant;

const SPAWNER_SIZE: usize = 2; // Must be a power of two, e.g., 2, 4, 8, 16, etc.
static TEST_EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_platform_time() -> Duration {
    let epoch = TEST_EPOCH.get_or_init(Instant::now);
    Instant::now().duration_since(*epoch)
}

fn main() {
    let spawner: ato::Spawner<SPAWNER_SIZE> = ato::Spawner::default();
    ato::spawn_task!(spawner, res, {
        let start = Instant::now();
        ato::sleep(Duration::from_millis(200), get_platform_time).await;
        let elapsed = Instant::now().duration_since(start);
        println!(
            "Task 0 completed after {:?} milliseconds",
            elapsed.as_millis()
        );
    });
    res.unwrap();

    spawner.run_until_all_done().unwrap();
}
