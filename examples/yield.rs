use std::sync::{Arc, Mutex};

use ato::{yield_now, Spawner};

const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.

fn main() {
    let spawner: Spawner<SPAWNER_SIZE> = Spawner::new();
    let lock = Arc::new(Mutex::new(Vec::new()));
    let lock_clone = lock.clone();
    let mut task = ato::task!({
        {
            let mut num = lock_clone.lock().unwrap();
            num.push(1);
        }
        yield_now().await; // Yield control back to the scheduler
        {
            let mut num = lock_clone.lock().unwrap();
            num.push(3);
        }
    });
    spawner.spawn(&mut task).unwrap();

    let lock_clone = lock.clone();
    let mut task_2 = ato::task!({
        {
            let mut num = lock_clone.lock().unwrap();
            num.push(2);
        }
    });
    spawner.spawn(&mut task_2).unwrap();

    spawner.run_until_all_done().unwrap();

    // check that the lock was accessed correctly
    let num = lock.lock().unwrap();
    assert_eq!(
        *num,
        Vec::from([1, 2, 3]),
        "Lock was not accessed correctly"
    );
    println!("Lock was accessed correctly: {:?}", *num);
}
