use console::style;
use indicatif::HumanBytes;
use std::time::Duration;

use rand::{RngCore, SeedableRng};

use crate::{
    structs::{lsm::tests::check_file, *},
    utils::{error::Result, new_progress_bar, DataStore},
};

#[test]
fn it_works() -> Result<()> {
    crate::core::runtime::block_on(it_works_async()) // ensure use one async runtime
}

const TEST_SIZE: u64 = 100000;
const RANDOM_TEST_SIZE: u64 = 200;
const FLUSH_INTERVAL: Duration = Duration::from_millis(700);
const ITER_COUNT: u64 = 98734;

const DATA_SIZE: usize = 240;
const NUMBER_TESTS: usize = 200;

async fn it_works_async() -> Result<()> {
    crate::utils::logger::init();
    let test_dir = "helper/table_test";

    std::fs::remove_dir_all(test_dir).ok();
    std::fs::create_dir_all(test_dir).unwrap();

    let table = Table::open(test_dir.to_string()).await?;

    assert_eq!(table.name(), test_dir);
    assert_eq!(table.id(), TableId::new(test_dir));

    init_table(&table).await;

    check_table_files(&table).await?;

    let iter_elapsed = iter_table(&table).await?;

    let seq_elapsed = seq_read_table(&table).await?;

    let rand_elapsed = rand_read_table(&table).await?;

    info!(
        "{:=^80}",
        style(format!(
            " Read Test Passed ({:?}/{:?}/{:?}) ",
            iter_elapsed, seq_elapsed, rand_elapsed
        ))
        .green()
    );

    let size_on_disk = table.size_on_disk().await?;

    info!(
        "Size on disk: {}",
        style(HumanBytes(size_on_disk).to_string()).green().bold()
    );

    assert_eq!(table.get(TEST_SIZE + 20).await?, DataStore::NotFound);

    info!("{:=^80}", style(" All Test Passed ").green());
    Ok(())
}

async fn init_table(table: &Table) {
    info!("{:=^80}", style(" Init Test Data ").yellow());
    let start = std::time::Instant::now();

    let bar = new_progress_bar(TEST_SIZE / 2);
    for i in 0..TEST_SIZE / 2 {
        // random with seed i
        let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
        bar.inc(1);
    }
    bar.finish();

    let bar = new_progress_bar(TEST_SIZE / 10);
    for i in (0..TEST_SIZE / 2).step_by(5) {
        table.delete(i).await;
        bar.inc(1);
    }
    bar.finish();

    info!(
        "{:=^80}",
        style(format!(" Init Test Data Done ({:?}) ", start.elapsed())).green()
    );

    info!("{}", style(">>> Waiting for flush...").cyan().bold());
    tokio::time::sleep(FLUSH_INTERVAL).await;

    info!("{:=^80}", style(" Add More Data ").yellow());
    let start = std::time::Instant::now();

    // update some data
    let bar = new_progress_bar(TEST_SIZE / 26);
    for i in (0..TEST_SIZE / 2).step_by(13) {
        // random with seed i
        let mut data = vec![((i * 2) % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
        bar.inc(1);
    }
    bar.finish();

    // delete some data
    let bar = new_progress_bar(TEST_SIZE / 29);
    for i in (0..TEST_SIZE).step_by(29) {
        table.delete(i).await;
        bar.inc(1);
    }
    bar.finish();

    // add more data
    let bar = new_progress_bar(TEST_SIZE / 2);
    for i in TEST_SIZE / 2..TEST_SIZE {
        // random with seed i
        let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

        let mut rng = rand::rngs::StdRng::seed_from_u64(i);
        let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
        rng.fill_bytes(&mut rnd_data);

        data.extend_from_slice(&rnd_data);
        table.set(i, data).await;
        bar.inc(1);
    }
    bar.finish();

    info!(
        "{:=^80}",
        style(format!(" Add More Data Done ({:?}) ", start.elapsed())).green()
    );

    info!("{}", style(">>> Waiting for flush...").cyan().bold());
    tokio::time::sleep(FLUSH_INTERVAL).await;
}

async fn check_table_files(table: &Table) -> Result<()> {
    info!("{:=^80}", style(" Check Files ").yellow());
    let start = std::time::Instant::now();

    for table in table.table_files().await {
        check_file(&table).await?;
    }

    info!(
        "{:=^80}",
        style(format!(" Check Files Done ({:?}) ", start.elapsed())).green()
    );

    Ok(())
}

async fn iter_table(table: &Table) -> Result<Duration> {
    info!("{:=^80}", style(" Iter Test ").yellow());

    let start = std::time::Instant::now();

    table.init_iter().await;

    let mut count = 0;
    let bar = new_progress_bar(ITER_COUNT);
    while let Some(next) = table.next().await? {
        trace!("Got next item: [{}] -> [{}]", next.0, next.1);
        count += 1;
        bar.inc(1);

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

        debug_assert_eq!(
            next.1.unwrap().as_ref(),
            &data,
            "Data mismatch for key {}",
            next.0
        );
    }
    bar.finish();

    table.end_iter().await;

    let elapsed = start.elapsed();

    info!(
        "{:=^80}",
        style(format!(" Got {} Items ({:?}) ", count, elapsed)).green()
    );

    Ok(elapsed)
}

async fn seq_read_table(table: &Table) -> Result<Duration> {
    info!(
        "{:=^80}",
        style(format!(" Sequential Read Test ({}) ", TEST_SIZE)).yellow()
    );
    let start = std::time::Instant::now();

    let bar = new_progress_bar(TEST_SIZE);
    for i in 0..TEST_SIZE {
        bar.inc(1);

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
    bar.finish();

    let elapsed = start.elapsed();
    info!(
        "{:=^80}",
        style(format!(" Sequential Read Test Done ({:?}) ", elapsed)).green()
    );

    Ok(elapsed)
}

async fn rand_read_table(table: &Table) -> Result<Duration> {
    info!(
        "{:=^80}",
        style(format!(" Random Read Test ({}) ", RANDOM_TEST_SIZE)).yellow()
    );

    let start = std::time::Instant::now();
    let mut rng = rand::rngs::StdRng::seed_from_u64(table.id().0);

    let bar = new_progress_bar(RANDOM_TEST_SIZE);
    for _ in 0..RANDOM_TEST_SIZE {
        let key = rng.next_u64() % TEST_SIZE;
        bar.inc(1);

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
    bar.finish();

    let elapsed = start.elapsed();
    info!(
        "{:=^80}",
        style(format!(" Random Read Test Done ({:?}) ", elapsed)).green()
    );

    Ok(elapsed)
}
