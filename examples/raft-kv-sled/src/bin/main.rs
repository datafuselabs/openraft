use clap::Parser;
use openraft::Raft;
use raft_key_value_sled::network::raft_network_impl::ExampleNetwork;
use raft_key_value_sled::start_example_raft_node;
use raft_key_value_sled::store::ExampleStore;
use raft_key_value_sled::ExampleTypeConfig;

pub type ExampleRaft = Raft<ExampleTypeConfig, ExampleNetwork, ExampleStore>;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    #[clap(long)]
    pub id: u64,

    #[clap(long)]
    pub http_addr: String,

    #[clap(long)]
    pub rpc_addr: String,
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    // Setup the logger
    tracing_subscriber::fmt().init();

    // Parse the parameters passed by arguments.
    let options = Opt::parse();
    let db_path = format!("{}.db", options.rpc_addr);

    start_example_raft_node(
        options.id,
        &db_path,
        options.http_addr,
        options.rpc_addr,
    )
    .await
}
