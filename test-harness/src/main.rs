mod model;
mod process;

use clap::Parser;
use tokio::time::Duration;
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

    let mut envoy = process::Process::new(process::Config {
        ads: false,
        delta: false,
        service_node: "lol".to_string(),
        service_cluster: "wat".to_string(),
    });
    envoy.spawn().expect("Failed to spawn");

    let config_dump = process::poll::until(
        process::poll::Config {
            timeout: Duration::from_secs(5),
            backoff: Duration::from_secs(1),
        },
        || async { process::get::config_dump(9901).await },
    )
    .await
    .expect("Failed to get config_dump");

    println!("{:?}", config_dump);
}
