
use std::sync::atomic::{AtomicUsize, Ordering};

use raft_key_value_sled::ExampleNodeId;
use raft_key_value_sled::store::ExampleStore;


const TEST_DATA_DIR: &str =  "./target/sled-test-db/"; // cargo test uses different base point then cargo run. Source: https://github.com/rust-lang/cargo/issues/8340
static GLOBAL_TEST_COUNT: AtomicUsize = AtomicUsize::new(0);

#[test]
pub fn test_raft_store() -> Result<(), openraft::StorageError<ExampleNodeId>> {
    let pid = std::process::id();
    let dir = format!("{}pid{}/", TEST_DATA_DIR, pid);
    let db_dir = std::path::Path::new(&dir);
    if db_dir.exists() {
        std::fs::remove_dir_all(&dir).expect("Could not prepare test directory");
    }

    let test_res = openraft::testing::Suite::test_all(test_store_factory);
    if db_dir.exists() {
        std::fs::remove_dir_all(&dir).expect("Could not prepare test directory");
    }

    test_res?;

    Ok(())
}

async fn test_store_factory() -> std::sync::Arc<ExampleStore> {
    let pid = std::process::id();
    let old_count = GLOBAL_TEST_COUNT.fetch_add(1, Ordering::SeqCst);
    let dir = format!("{}pid{}/num{}/", TEST_DATA_DIR, pid, old_count);
    ExampleStore::new(&dir).await
}
