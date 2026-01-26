/// Outputs:
/// Sending: 0
/// Received: 0
/// Sending: 1
/// Received: 1
/// Sending: 2
/// Received: 2
/// Sending: 3
/// Received: 3
/// Sending: 4
/// Received: 4
fn main() {
    let spawner: ato::Spawner<2> = ato::Spawner::default();
    ato::channel!(hello, u32, 2); // Must be a power of two, e.g., 2, 4, 8, 16, etc.

    // Consumer
    let consumer = hello.1.clone();
    ato::spawn!(spawner, async {
        for _ in 0..5 {
            let msg = consumer.recv().await;
            println!("Received: {}", msg);
        }
    });

    // Producer
    ato::spawn!(spawner, async {
        for i in 0..5 {
            println!("Sending: {}", i);
            hello.0.send(i).await;
            ato::yield_now().await;
        }
    });

    spawner.run_until_all_done().unwrap();
}
