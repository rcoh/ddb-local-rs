# Adding a New Operation to ddb-local-rs

This guide walks through adding support for a new DynamoDB operation. We'll use `CreateTable` as the example.

## Overview

Adding a new operation involves four steps:
1. Update the Python extraction script to include the new operation
2. Regenerate the Smithy model
3. Regenerate the Rust server SDK
4. Implement the operation in the backend

## Step 1: Update the Extraction Script

Edit `smithy/extract_minimal.py` to add your operation to the list:

```python
# Start with GetItem, PutItem, and CreateTable
needed = set()
find_deps('com.amazonaws.dynamodb#GetItem', needed)
find_deps('com.amazonaws.dynamodb#PutItem', needed)
find_deps('com.amazonaws.dynamodb#CreateTable', needed)  # Add this line
```

Also update the service operations list:

```python
output.append('    operations: [')
output.append('        GetItem')
output.append('        PutItem')
output.append('        CreateTable')  # Add this line
output.append('    ]')
```

## Step 2: Regenerate the Smithy Model

Run the extraction script to generate the updated Smithy model:

```bash
cd smithy
python3 extract_minimal.py > model/main.smithy
```

This reads from `model/dynamo.json.bak` (the full AWS DynamoDB model) and extracts only the shapes needed for your operations.

## Step 3: Regenerate the Server SDK

Build the Smithy model to generate the Rust server SDK:

```bash
cd ..
./gradlew build
```

This runs the `rust-server-codegen` plugin which generates:
- Input/output structures in `server-sdk/src/input.rs` and `server-sdk/src/output.rs`
- Error types in `server-sdk/src/error.rs`
- Operation handlers in `server-sdk/src/operation.rs`

## Step 4: Implement the Backend

### 4.1 Add to the DynamoDb Trait

Edit `ddb-local/src/lib.rs` and add the method signature to the `DynamoDb` trait:

```rust
#[async_trait::async_trait]
pub trait DynamoDb: Send + Sync {
    // ... existing methods ...
    
    async fn create_table(
        &self,
        input: input::CreateTableInput,
    ) -> Result<output::CreateTableOutput, error::CreateTableError>;
}
```

### 4.2 Wire Up in the Service Builder

In the `build_service!` macro in `ddb-local/src/lib.rs`, add the handler:

```rust
macro_rules! build_service {
    ($backend:expr) => {{
        // ... config setup ...
        
        let get_backend = $backend.clone();
        let put_backend = $backend.clone();
        let create_table_backend = $backend.clone();  // Add this
        
        DynamoDb20120810::builder(config)
            .get_item(move |input| {
                let backend = get_backend.clone();
                async move { backend.get_item(input).await }
            })
            .put_item(move |input| {
                let backend = put_backend.clone();
                async move { backend.put_item(input).await }
            })
            .create_table(move |input| {  // Add this block
                let backend = create_table_backend.clone();
                async move { backend.create_table(input).await }
            })
            .build()
            .expect("failed to build DynamoDB service")
    }};
}
```

### 4.3 Implement in the Backend

Edit `ddb-local/src/backend.rs` and implement the operation in the `impl DynamoDb for InMemoryDynamoDb` block:

```rust
async fn create_table(
    &self,
    input: input::CreateTableInput,
) -> Result<output::CreateTableOutput, error::CreateTableError> {
    let key_schema: Vec<String> = input
        .key_schema
        .iter()
        .map(|k| k.attribute_name.clone())
        .collect();

    match self.store.lock().unwrap().entry(input.table_name.clone()) {
        Entry::Vacant(v) => {
            v.insert(TableStore {
                schema: key_schema,
                items: HashMap::new(),
            });
            Ok(output::CreateTableOutput {
                table_description: None,
            })
        }
        Entry::Occupied(_) => Err(error::CreateTableError::ResourceInUseException(
            error::ResourceInUseException::builder()
                .message(Some(format!("Table {} already exists", input.table_name)))
                .build(),
        )),
    }
}
```

## Step 5: Test

Add tests in `ddb-local/src/backend.rs`:

```rust
#[tokio::test]
async fn test_create_table() {
    let (client, _store) = create_in_memory_dynamodb_client().await;

    let result = client
        .create_table()
        .table_name("new-table")
        .key_schema(
            aws_sdk_dynamodb::types::KeySchemaElement::builder()
                .attribute_name("id")
                .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                .build()
                .unwrap(),
        )
        .attribute_definitions(
            aws_sdk_dynamodb::types::AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()
                .unwrap(),
        )
        .send()
        .await;

    assert!(result.is_ok());
}
```

Run the tests:

```bash
cargo test
```

## Common Issues

### Missing Fields in Output Structures

If you get errors like `missing field 'consumed_capacity'`, check the generated output structure and make sure you're setting all fields (even if to `None`).

### ValidationException Not in Errors

The extraction script automatically adds `ValidationException` to all operations. If you see warnings about constrained inputs, this is already handled.

### Service Doesn't Implement Clone

Use the `build_service!` macro pattern shown above. Don't try to return the service directly from a function - the macro handles the generic types correctly.

## Tips

- The full AWS DynamoDB model is in `smithy/model/dynamo.json.bak` - search it to understand structure shapes
- Generated code is in `server-sdk/src/` - look there to see what types are available
- The `@error` trait is automatically added to error structures by the extraction script
- Use `cargo build` to check for compilation errors after each step
