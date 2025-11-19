# ddb-local

A local DynamoDB implementation in Rust for testing. Built with [Smithy](https://smithy.io/).

This crate provides both an in-memory client (no network) and a network server implementation of DynamoDB. Perfect for integration tests without hitting AWS.

## Features

- In-memory transport (no network overhead)
- Network server mode
- Compatible with the AWS SDK for Rust
- Currently supports `GetItem` and `PutItem` operations

## Quick Start

### In-memory testing (recommended)

```rust
use ddb_local::DynamoDbLocal;
use aws_sdk_dynamodb::types::AttributeValue;

#[tokio::test]
async fn test_dynamodb() {
    let local = DynamoDbLocal::builder().as_http_client();
    let client = local.client().await;

    client.put_item()
        .table_name("test-table")
        .item("id", AttributeValue::S("123".into()))
        .item("name", AttributeValue::S("test".into()))
        .send()
        .await
        .unwrap();

    let result = client.get_item()
        .table_name("test-table")
        .key("id", AttributeValue::S("123".into()))
        .send()
        .await
        .unwrap();

    assert!(result.item.is_some());
}
```

### Network server

```rust
use ddb_local::DynamoDbLocal;

#[tokio::main]
async fn main() {
    // Bind to an automatically assigned port
    let local = DynamoDbLocal::builder()
        .bind()
        .await
        .unwrap();

    println!("DynamoDB running at {}", local.endpoint_url());

    // Get a pre-configured client
    let client = local.client().await;
    
    // Use the client...
}
```

Or bind to a specific address:

```rust
let local = DynamoDbLocal::builder()
    .bind_to_address("127.0.0.1:8000")
    .await
    .unwrap();
```

## Custom Backend

You can implement your own backend by implementing the `DynamoDb` trait:

```rust
use ddb_local::{DynamoDb, DynamoDbLocal};

struct MyBackend;

#[async_trait::async_trait]
impl DynamoDb for MyBackend {
    async fn get_item(&self, input: input::GetItemInput) 
        -> Result<output::GetItemOutput, error::GetItemError> {
        // Your implementation
    }

    async fn put_item(&self, input: input::PutItemInput) 
        -> Result<output::PutItemOutput, error::PutItemError> {
        // Your implementation
    }
}

let local = DynamoDbLocal::builder()
    .with_backend(MyBackend)
    .as_http_client();
```

## Standalone Server

Run as a standalone server:

```bash
cargo install ddb-local
ddb-local
```

This starts a server on `localhost:8888` that you can use with the AWS CLI or any AWS SDK.

## License

MIT
