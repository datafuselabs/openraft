use std::sync::Arc;
use std::time::Instant;

use maplit::btreeset;
use openraft::Config;
use tokio::runtime::Builder;

use crate::fixtures::RaftRouter;

#[test]
#[ignore]
fn bench_cluster() -> anyhow::Result<()> {
    // configs

    let worker_threads = 8;
    let n_operations = 10_000;

    let rt = Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .thread_name("bench-cluster")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();

    // Run client_write benchmark
    let output = rt.block_on(foo(n_operations))?;
    Ok(output)
}

/// Benchmkar client_write.
///
/// Cluster config:
/// - Members: 3
/// - Learners: 0
/// - Log: in-memory BTree
/// - StateMachine: in-memory BTree
async fn foo(n: u64) -> anyhow::Result<()> {
    let config = Arc::new(
        Config {
            election_timeout_min: 200,
            election_timeout_max: 2000,
            ..Default::default()
        }
        .validate()?,
    );
    let router = Arc::new(RaftRouter::new(config.clone()));
    let _log_index = router.new_nodes_from_single(btreeset! {0,1,2}, btreeset! {}).await?;

    let now = Instant::now();

    for i in 0..n {
        router.client_request(0, "foo", i as u64).await
    }

    let elapsed = now.elapsed();

    println!(
        "n={}, time: {:?}, ns/op: {}, op/ms: {}",
        n,
        elapsed,
        elapsed.as_nanos() / (n as u128),
        (n as u128) / elapsed.as_millis(),
    );

    Ok(())
}
