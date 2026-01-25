use core::time::Duration;
use std::time::Instant;

static TEST_EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_platform_time() -> Duration {
    let epoch = TEST_EPOCH.get_or_init(Instant::now);
    Instant::now().duration_since(*epoch)
}

fn main() {
    let spawner: ato::Spawner<2> = ato::Spawner::default();
    let start_time = get_platform_time();
    // Run interval task for 3 iterations (500 ms), with sleep in between, which
    // should still be 1500 ms total.
    ato::spawn!(spawner, async {
        let mut interval =
            ato::interval::Interval::new(Duration::from_millis(500), get_platform_time);
        for _ in 0..3 {
            interval.tick().await;
            // sleep for random duration less than the interval
            ato::sleep(Duration::from_millis(200), get_platform_time).await;
        }
    });
    assert!(spawner.run_until_all_done().is_ok());

    let elapsed = get_platform_time() - start_time;
    assert!(elapsed <= Duration::from_millis(1550)); // with some margin
}
