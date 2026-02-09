# Testing Against DynamoDB Local

This project supports running tests against both the in-memory implementation and real AWS DynamoDB Local to validate compatibility.

## Test Architecture

Tests use `rstest` for parameterized testing with two backends:
- `TestBackendType::InMemory` - Our implementation
- `TestBackendType::DynamoDbLocal` - Real DynamoDB Local

## Writing Parameterized Tests

Use `rstest` to run tests against both backends:

```rust
use rstest::rstest;

#[rstest]
#[case::in_memory(TestBackendType::InMemory)]
#[case::dynamodb_local(TestBackendType::DynamoDbLocal)]
#[tokio::test]
async fn test_my_operation(#[case] backend_type: TestBackendType) {
    let (client, backend) = create_test_client(backend_type).await;
    backend.create_table("test-table", &["id"]);
    
    // Your test code here using the client
}
```

## Running Tests

### Against In-Memory Only (default)

```bash
cargo test
```

This runs all tests, but DynamoDB Local tests will fail if no server is running.

### Against In-Memory Only (filtered)

```bash
cargo test in_memory
```

This runs only the in-memory test cases.

### Against DynamoDB Local

1. Start DynamoDB Local:
```bash
docker run -p 8000:8000 amazon/dynamodb-local
```

2. Run all tests:
```bash
cargo test
```

Or run only DynamoDB Local tests:
```bash
cargo test dynamodb_local
```

### Custom DynamoDB Local Endpoint

Set the `DYNAMODB_LOCAL_ENDPOINT` environment variable:

```bash
DYNAMODB_LOCAL_ENDPOINT=http://localhost:9000 cargo test
```

Default is `http://localhost:8000`.

## Test Behavior Differences

### Table Creation

- **InMemory**: Uses `backend.create_table()` helper for fast setup
- **DynamoDbLocal**: Must use the client's `create_table()` API

The `TestBackend::create_table()` method handles this automatically:
- For InMemory: Creates table directly in the backend
- For DynamoDbLocal: No-op (table must be created via client API)

### Test Isolation

- **InMemory**: Each test gets a fresh backend instance
- **DynamoDbLocal**: Tests share the same server instance

For DynamoDB Local, use unique table names per test to avoid conflicts:

```rust
let table_name = format!("test-table-{}", uuid::Uuid::new_v4());
```

## Converting Existing Tests

To convert a test to run against both backends:

1. Add `rstest` attributes:
```rust
#[rstest]
#[case::in_memory(TestBackendType::InMemory)]
#[case::dynamodb_local(TestBackendType::DynamoDbLocal)]
#[tokio::test]
async fn test_name(#[case] backend_type: TestBackendType) {
```

2. Change client creation:
```rust
// Before:
let (client, store) = create_in_memory_dynamodb_client().await;
store.create_table("test-table", &["id"]);

// After:
let (client, backend) = create_test_client(backend_type).await;
backend.create_table("test-table", &["id"]);
```

3. Remove any in-memory specific code (like accessing `store` internals)

## CI/CD Integration

In CI, you can:

1. Run in-memory tests only (fast):
```bash
cargo test in_memory
```

2. Run both with DynamoDB Local in Docker:
```bash
docker run -d -p 8000:8000 amazon/dynamodb-local
cargo test
```

## Limitations

Not all tests can run against DynamoDB Local:
- Tests that inspect internal state (use `create_in_memory_dynamodb_client()` directly)
- Tests for features not yet in DynamoDB Local
- Tests that require specific error conditions

Keep these as in-memory only tests without the `rstest` parameterization.
