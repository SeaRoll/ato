const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.

fn main() {
    // create a spawner with the specified size
    let spawner: ato::Spawner<SPAWNER_SIZE> = ato::Spawner::default();

    // create a simple task that prints a message
    ato::task!(task, {
        println!("Hello, World!");
    });
    spawner.spawn(task).unwrap();

    // run until all tasks are done running
    spawner.run_until_all_done().unwrap();
}
