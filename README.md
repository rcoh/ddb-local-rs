# ddb-local-rs

A local DynamoDB implementation in Rust, built with [Smithy](https://smithy.io/). Useful for testing without hitting AWS.

Currently supports `GetItem` and `PutItem` operations with an in-memory backend.

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
ddb-local = { git = "https://github.com/rcoh/ddb-local-rs" }
```

### In-memory (no network)

```rust
use ddb_local::DynamoDbLocal;
use aws_sdk_dynamodb::types::AttributeValue;

#[tokio::test]
async fn test_dynamodb() {
    let local = DynamoDbLocal::builder().as_http_client();
    let client = local.client().await;

    // Use the client like normal
    client.put_item()
        .table_name("test-table")
        .item("id", AttributeValue::S("123".into()))
        .send()
        .await
        .unwrap();
}
```

### Network server

```rust
use ddb_local::DynamoDbLocal;

#[tokio::main]
async fn main() {
    let local = DynamoDbLocal::builder()
        .bind()
        .await
        .unwrap();

    println!("DynamoDB running at {}", local.endpoint_url());

    // Get a pre-configured client
    let client = local.client().await;
}
```

Or bind to a specific address:

```rust
let local = DynamoDbLocal::builder()
    .bind_to_address("127.0.0.1:8000")
    .await
    .unwrap();
```

## Building

The project uses Smithy to generate the server SDK from the model:

```bash
./gradlew build
cargo build
```

## Running the standalone server

```bash
cargo run --bin ddb-local
```

This starts a server on `localhost:8888` that you can hit with the AWS SDK or CLI.
