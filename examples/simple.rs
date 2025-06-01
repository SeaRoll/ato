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
