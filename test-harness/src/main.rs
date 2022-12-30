mod model;

use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Whether to configure client Envoys (and the cache) to use ADS.
    #[arg(short, long, default_value_t = false)]
    ads: bool,

    /// Whether to configure client Envoys to use the state of the world (default), or incremental
    /// delta gRPC protocol.
    #[arg(short, long, default_value_t = false)]
    delta: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    info!("Running with args={:?}", args);
}
