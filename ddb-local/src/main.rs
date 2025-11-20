use clap::Parser;
use tracing_subscriber::{EnvFilter, prelude::*};

pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8888;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, action, default_value = DEFAULT_ADDRESS)]
    address: String,
    #[clap(short, long, action, default_value_t = DEFAULT_PORT)]
    port: u16,
}

pub fn setup_tracing() {
    let format = tracing_subscriber::fmt::layer();
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(format)
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    setup_tracing();

    let bind = format!("{}:{}", args.address, args.port);
    let local = ddb_local::DynamoDbLocal::builder()
        .bind_to_address(bind.parse().expect("unable to parse bind address"))
        .await
        .expect("failed to bind server");

    tracing::info!("server listening on {}", local.addr());

    // Keep the server running
    tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
}
