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
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    match args.name.as_str() {
        "test1" => {
            let mut test1 = Test::new(tests::test1::init());
            test1.run(tests::test1::test).await;
        }
        "test2" => {
            let mut test2 = Test::new(tests::test2::init());
            test2.run(tests::test2::test).await;
        }
        "test3" => {
            let mut test3 = Test::new(tests::test3::init());
            test3.run(tests::test3::test).await;
        }
        _ => log::error!("Unknown test name {}", args.name),
    }
}
