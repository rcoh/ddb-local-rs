# Testing Design

## Overview

The test suite supports running tests against two backends:
1. **In-memory implementation** - Fast, isolated, default
2. **Real DynamoDB Local** - Validates compatibility with AWS

## Architecture

### Backend Abstraction

```rust
#[derive(Debug, Clone, Copy)]
pub enum TestBackendType {
    InMemory,
    DynamoDbLocal,
}

pub struct TestBackend {
    backend_type: TestBackendType,
    in_memory: Option<InMemoryDynamoDb>,
}
```

### Client Creation

```rust
pub async fn create_test_client(backend_type: TestBackendType) -> (Client, TestBackend)
```

This function:
- For `InMemory`: Creates an in-process client with no network
- For `DynamoDbLocal`: Creates a client pointing to `http://localhost:8000` (or `DYNAMODB_LOCAL_ENDPOINT`)

### Parameterized Tests with rstest

Tests use `rstest` to run the same test against both backends:

```rust
#[rstest]
#[case::in_memory(TestBackendType::InMemory)]
#[case::dynamodb_local(TestBackendType::DynamoDbLocal)]
#[tokio::test]
async fn test_operation(#[case] backend_type: TestBackendType) {
    let (client, backend) = create_test_client(backend_type).await;
    // Test code
}
```

This generates two test cases:
- `test_operation::case_1_in_memory`
- `test_operation::case_2_dynamodb_local`

## Test Categories

### 1. Parameterized Tests (Both Backends)

Tests that validate API compatibility. These should:
- Use `create_test_client(backend_type)`
- Only interact via the AWS SDK client
- Not inspect internal state
- Use `backend.create_table()` for setup

Example: `test_put_and_get_item`

### 2. In-Memory Only Tests

Tests that:
- Inspect internal backend state
- Test implementation-specific behavior
- Test features not in DynamoDB Local

These use `create_in_memory_dynamodb_client()` directly.

Example: `test_multiple_clients_same_store`

### 3. Ignored Tests

Tests for unimplemented features marked with `#[ignore]`.

## Running Tests

```bash
# All in-memory tests (fast)
cargo test in_memory

# All tests (requires DynamoDB Local for full pass)
cargo test

# Only DynamoDB Local tests
cargo test dynamodb_local
```

## Design Decisions

### Why rstest?

- Clean parameterization syntax
- Each backend gets a separate test case
- Easy to filter by backend type
- Good error messages

### Why not a test fixture?

- Async fixtures are complex in Rust
- rstest's case-based approach is simpler
- Each test explicitly chooses its backend

### Why keep two client creation functions?

- `create_test_client()` - For parameterized tests
- `create_in_memory_dynamodb_client()` - For in-memory specific tests

This allows gradual migration and keeps in-memory-only tests simple.

### Table Creation Abstraction

`TestBackend::create_table()` handles the difference:
- **InMemory**: Direct backend call (fast)
- **DynamoDbLocal**: No-op (use client API)

This keeps test code clean while handling backend differences.

## Migration Strategy

1. Start with one parameterized test as proof of concept
2. Gradually convert tests that don't need internal state access
3. Keep in-memory-only tests for implementation details
4. Document which tests should remain in-memory only

## Future Enhancements

- Automatic table cleanup for DynamoDB Local tests
- Parallel test execution with unique table names
- CI integration with DynamoDB Local container
- Performance comparison between implementations
