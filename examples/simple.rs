use ato::Spawner;

const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.

fn main() {
    // create a spawner with the specified size
    let spawner: Spawner<SPAWNER_SIZE> = Spawner::default();

    // create a simple task that prints a message
    let mut task = ato::task!({
        println!("Task 1 started");
    });
    spawner.spawn(&mut task).unwrap();

    // run until all tasks are done running
    spawner.run_until_all_done().unwrap();
}
