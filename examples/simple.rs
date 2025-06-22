use ato::Spawner;

const SPAWNER_SIZE: usize = 4; // Must be a power of two, e.g., 2, 4, 8, 16, etc.
static SPAWNER: Spawner<SPAWNER_SIZE> = Spawner::new();

fn main() {
    SPAWNER
        .spawn(async {
            println!("Task 1 started");
        })
        .unwrap();

    // Spawn another task
    SPAWNER
        .spawn(async {
            println!("Task 2 started, spawning another task");
            SPAWNER
                .spawn(async {
                    println!("Task 2.1 started");
                })
                .unwrap();
            println!("Task 2 completed");
        })
        .unwrap();

    SPAWNER.run_until_all_done().unwrap();
}
