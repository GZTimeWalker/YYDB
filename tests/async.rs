#[test]
#[cfg(test)]
fn test_spawn() {
    yengine::run_sync(async {
        let mut futures = vec![];
        for i in 0..5 {
            futures.push(yengine::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100 * (i + 1))).await;
                println!("Y-Engine Async Spawn Test: {}", i);
            }));
        }

        yengine::wait_all(futures).await.unwrap();
    });
}

#[test]
#[cfg(test)]
fn test_run() {
    yengine::run_sync(async {
        println!("Y-Engine Async Test...");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        println!("Y-Engine Async Test End.");
    });
}
