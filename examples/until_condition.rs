use std::sync::atomic::AtomicBool;

const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.
static TASK_DONE: AtomicBool = AtomicBool::new(false);

fn main() {
    // create a spawner with the specified size
    let spawner: ato::Spawner<SPAWNER_SIZE> = ato::Spawner::default();

    // create a simple task that prints a message
    ato::task!(task, {
        let mut i = 0;
        while i < 5 {
            println!("Running {}", i);
            i += 1;
        }
        TASK_DONE.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    spawner.spawn(task).unwrap();

    loop {
        if TASK_DONE.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        spawner.run_once().unwrap();
    }

    println!("Task completed.");
    assert!(TASK_DONE.load(std::sync::atomic::Ordering::SeqCst));
}
