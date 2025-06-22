use std::sync::{Arc, Mutex};

use ato::{yield_now, Spawner};

fn main() {
    // Creates a spawner.
    // NOTE: The size must be power of two, e.g., 2, 4, 8, 16, etc.
    static SPAWNER: Spawner<8> = Spawner::new();

    let lock = Arc::new(Mutex::new(Vec::new()));

    let lock_clone = lock.clone();
    SPAWNER
        .spawn(async move {
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(1);
            }
            yield_now().await; // Yield control back to the scheduler
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(3);
            }
        })
        .unwrap();

    let lock_clone = lock.clone();
    SPAWNER
        .spawn(async move {
            {
                let mut num = lock_clone.lock().unwrap();
                num.push(2);
            }
        })
        .unwrap();

    SPAWNER.run_until_all_done().unwrap();

    // check that the lock was accessed correctly
    let num = lock.lock().unwrap();
    assert_eq!(
        *num,
        Vec::from([1, 2, 3]),
        "Lock was not accessed correctly"
    );
}
