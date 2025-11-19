use clap::Parser;
use dynamodb_local_server_sdk::server::{
    instrumentation::InstrumentExt,
    layer::alb_health_check::AlbHealthCheckLayer,
    plugin::{HttpPlugins, ModelPlugins},
    request::request_id::ServerRequestIdProviderLayer,
};
use hyper::StatusCode;
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, prelude::*};

use dynamodb_local_server_sdk::{DynamoDb20120810, DynamoDb20120810Config};

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

    let http_plugins = HttpPlugins::new().instrument();
    let model_plugins = ModelPlugins::new();

    let config = DynamoDb20120810Config::builder()
        .layer(AlbHealthCheckLayer::from_handler("/ping", |_req| async {
            StatusCode::OK
        }))
        .layer(ServerRequestIdProviderLayer::new())
        .http_plugin(http_plugins)
        .model_plugin(model_plugins)
        .build();

    let app = DynamoDb20120810::builder(config)
        .get_item(ddb_local::get_item)
        .put_item(ddb_local::put_item)
        .build()
        .expect("failed to build DynamoDB service");

    let make_app = app.into_make_service_with_connect_info::<SocketAddr>();

    let bind: SocketAddr = format!("{}:{}", args.address, args.port)
        .parse()
        .expect("unable to parse bind address");
    let server = hyper::Server::bind(&bind).serve(make_app);

    tracing::info!("server listening on {bind}");

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}
