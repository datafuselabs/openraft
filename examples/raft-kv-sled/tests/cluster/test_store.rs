
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::future::Future;

use async_trait::async_trait;

use openraft::StorageError;
use openraft::testing::StoreBuilder;

use raft_key_value_sled::ExampleNodeId;
use raft_key_value_sled::ExampleTypeConfig;
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

    let test_res = openraft::testing::Suite::test_all(ExampleBuilder{});
    if db_dir.exists() {
        std::fs::remove_dir_all(&dir).expect("Could not prepare test directory");
    }

    test_res?;

    Ok(())
}

struct ExampleBuilder {}
#[async_trait]
impl StoreBuilder<ExampleTypeConfig, Arc<ExampleStore>> for ExampleBuilder {
    async fn run_test<Fun, Ret, Res>(&self, t: Fun) -> Result<Ret, StorageError<ExampleNodeId>>
        where
            Res: Future<Output = Result<Ret, StorageError<ExampleNodeId>>> + Send,
            Fun: Fn(Arc<ExampleStore>) -> Res + Sync + Send,
    {
        let pid = std::process::id();
        let old_count = GLOBAL_TEST_COUNT.fetch_add(1, Ordering::SeqCst);
        let db_dir_str = format!("{}pid{}/num{}/", TEST_DATA_DIR, pid, old_count);

        let db_dir = std::path::Path::new(&db_dir_str);
        if !db_dir.exists() {
            std::fs::create_dir_all(db_dir).expect(&format!("could not create: {:?}", db_dir.to_str()))
        }

        let db: sled::Db = sled::open(db_dir)
            .expect(&format!("could not open: {:?}", db_dir.to_str()));

        let r = {
            let store = ExampleStore::new(std::sync::Arc::new(db)).await;
            t(store).await
        };
        r
    }
}
