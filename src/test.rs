use rand::{RngCore, SeedableRng};

use crate::{
    structs::{lsm::tests::check_file, *},
    utils::{error::Result, DataStore, human_read_size},
};

#[test]
fn it_works() -> Result<()> {
    crate::core::runtime::block_on(it_works_async()) // ensure use one async runtime
}

async fn it_works_async() -> Result<()> {
    crate::utils::logger::init();
    let test_dir = "helper/table_test";

    std::fs::remove_dir_all(test_dir).ok();
    std::fs::create_dir_all(test_dir).unwrap();

    let start = std::time::Instant::now();
    let table = Table::open(test_dir.to_string()).await?;

    assert_eq!(table.name(), test_dir);
    assert_eq!(table.id(), TableId::new(test_dir));

    const TEST_SIZE: u64 = 16000;
    const RANDOM_TEST_SIZE: usize = 2000;

    const DATA_SIZE: usize = 240;
    const NUMBER_TESTS: usize = 200;

    debug!("{:=^80}", " Init Test Data ");

    for i in 0..TEST_SIZE / 2 {
        // random with seed i
        let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
    }

    for i in (0..TEST_SIZE / 2).step_by(5) {
        table.delete(i).await;
    }

    debug!(
        "{:=^80}",
        format!(" Init Test Data Done ({:?}) ", start.elapsed())
    );

    debug!(">>> Waiting for flush...");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    debug!("{:=^80}", " Add More Data ");

    // update some data
    for i in (0..TEST_SIZE / 2).step_by(13) {
        // random with seed i
        let mut data = vec![((i * 2) % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
    }

    // delete some data
    for i in (0..TEST_SIZE).step_by(29) {
        table.delete(i).await;
    }

    // add more data
    for i in TEST_SIZE / 2..TEST_SIZE {
        // random with seed i
        let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
    }

    debug!(
        "{:=^80}",
        format!(" Add More Data Done ({:?}) ", start.elapsed())
    );

    debug!(">>> Waiting for flush...");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // check files
    debug!("{:=^80}", " Check Files ");
    let test_start = std::time::Instant::now();

    for table in table.table_files().await {
        check_file(&table).await?;
    }

    debug!(
        "{:=^80}",
        format!(" Check Files Done ({:?}) ", test_start.elapsed())
    );

    debug!("{:=^80}", format!(" Sequential Read Test ({}) ", TEST_SIZE));
    let start = std::time::Instant::now();

    for i in 0..TEST_SIZE {
        // random with seed i
        match table.get(i).await? {
            DataStore::Value(v) => {
                let mut data = if i % 29 == 0 && i <= TEST_SIZE / 2 {
                    vec![((i * 2) % 57 + 65) as u8; NUMBER_TESTS]
                } else {
                    vec![(i % 57 + 65) as u8; NUMBER_TESTS]
                };

                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
                rng.fill_bytes(&mut rnd_data);

                data.extend_from_slice(&rnd_data);

                debug_assert_eq!(v.as_ref(), &data, "Data mismatch for key {}", i);
            }
            x => {
                if i % 5 == 0 || i % 29 == 0 {
                    continue;
                } else {
                    panic!("Unexpected value for key {}: {:?}", i, x);
                }
            }
        }
    }

    debug!(
        "{:=^80}",
        format!(" Sequential Read Test Done ({:?}) ", start.elapsed())
    );

    debug!("{:=^80}", " Iter Test ");

    let start = std::time::Instant::now();

    table.init_iter().await;

    let mut count = 0;
    while let Some(next) = table.next().await? {
        trace!("Got next item: [{}] -> [{}]", next.0, next.1);
        count += 1;

        // do check
        let mut data = if next.0 % 29 == 0 && next.0 <= TEST_SIZE / 2 {
            vec![((next.0 * 2) % 57 + 65) as u8; NUMBER_TESTS]
        } else {
            vec![(next.0 % 57 + 65) as u8; NUMBER_TESTS]
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(next.0);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];

        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);

        debug_assert_eq!(next.1.unwrap().as_ref(), &data, "Data mismatch for key {}", next.0);
    }

    table.end_iter().await;

    debug!(
        "{:=^80}",
        format!(" Got {} Items ({:?}) ", count, start.elapsed())
    );

    // test for random key reading
    debug!(
        "{:=^80}",
        format!(" Random Read Test ({}) ", RANDOM_TEST_SIZE)
    );

    let start = std::time::Instant::now();

    for _ in 0..RANDOM_TEST_SIZE {
        let mut rng = rand::rngs::StdRng::seed_from_u64(rand::random());
        let key = rng.next_u64() % TEST_SIZE;

        match table.get(key).await? {
            DataStore::Value(v) => {
                let mut data = if key % 13 == 0 && key <= TEST_SIZE / 2 {
                    vec![((key * 2) % 57 + 65) as u8; NUMBER_TESTS]
                } else {
                    vec![(key % 57 + 65) as u8; NUMBER_TESTS]
                };

                let mut rng = rand::rngs::StdRng::seed_from_u64(key);
                let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
                rng.fill_bytes(&mut rnd_data);

                data.extend_from_slice(&rnd_data);

                debug_assert_eq!(v.as_ref(), &data, "Data mismatch for key {}", key);
            }
            x => {
                if key % 5 == 0 || key % 29 == 0 {
                    continue;
                } else {
                    panic!("Unexpected value for key {}: {:?}", key, x);
                }
            }
        }
    }

    debug!(
        "{:=^80}",
        format!(" Random Read Test Done ({:?}) ", start.elapsed())
    );

    debug!("{:=^80}", " NotFound Test ");

    assert_eq!(table.get(TEST_SIZE + 20).await?, DataStore::NotFound);

    let size_on_disk = table.size_on_disk().await?;

    debug!("Size on disk: {}", human_read_size(size_on_disk));

    debug!(
        "{:=^80}",
        format!(" All Test Passed ({:?}) ", test_start.elapsed())
    );
    Ok(())
}
