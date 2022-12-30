mod event;
mod generate;
mod model;
mod process;
mod test;
mod tests;

use test::Test;

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Name of the test to run
    #[arg(short, long)]
    name: String,

    /// Whether to configure Envoy (and the cache) to use ADS
    #[arg(short, long, default_value_t = false)]
    ads: bool,

    /// Whether to configure Envoy to use deltas.
    #[arg(short, long, default_value_t = false)]
    delta: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    match args.name.as_str() {
        "test1" => {
            let mut test1 = Test::new(tests::test1::init(), args.ads).await;
            test1.run(tests::test1::test, args.ads, args.delta).await;
        }
        "test2" => {
            let mut test2 = Test::new(tests::test2::init(), args.ads).await;
            test2.run(tests::test2::test, args.ads, args.delta).await;
        }
        "test3" => {
            let mut test3 = Test::new(tests::test3::init(), args.ads).await;
            test3.run(tests::test3::test, args.ads, args.delta).await;
        }
        _ => tracing::error!("Unknown test name {}", args.name),
    }
}
