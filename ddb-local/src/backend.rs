use crate::DynamoDb;
use aws_sdk_dynamodb::Client;
use dynamodb_local_server_sdk::{error, input, output};
use std::collections::{HashMap, hash_map::Entry};
use std::sync::{Arc, Mutex, MutexGuard};

fn evaluate_condition_expression(
    expression: &str,
    item: Option<&HashMap<String, dynamodb_local_server_sdk::model::AttributeValue>>,
    expression_attribute_values: Option<
        &HashMap<String, dynamodb_local_server_sdk::model::AttributeValue>,
    >,
) -> bool {
    let expr = expression.trim();

    // Handle AND expressions
    if expr.contains(" AND ") {
        return expr.split(" AND ").all(|sub_expr| {
            evaluate_condition_expression(sub_expr.trim(), item, expression_attribute_values)
        });
    }

    // Handle OR expressions
    if expr.contains(" OR ") {
        return expr.split(" OR ").any(|sub_expr| {
            evaluate_condition_expression(sub_expr.trim(), item, expression_attribute_values)
        });
    }

    // Handle attribute_not_exists(attr)
    if let Some(attr_start) = expr.find("attribute_not_exists(") {
        let attr_end = expr[attr_start..].find(')').unwrap() + attr_start;
        let attr_name = &expr[attr_start + 21..attr_end];
        return item.map_or(true, |i| !i.contains_key(attr_name));
    }

    // Handle attribute_exists(attr)
    if let Some(attr_start) = expr.find("attribute_exists(") {
        let attr_end = expr[attr_start..].find(')').unwrap() + attr_start;
        let attr_name = &expr[attr_start + 17..attr_end];
        return item.map_or(false, |i| i.contains_key(attr_name));
    }

    // Handle equality: attr = :val
    if let Some(eq_pos) = expr.find(" = ") {
        let attr_name = expr[..eq_pos].trim();
        let value_ref = expr[eq_pos + 3..].trim();

        if let (Some(item), Some(values)) = (item, expression_attribute_values) {
            if let (Some(item_value), Some(expected_value)) =
                (item.get(attr_name), values.get(value_ref))
            {
                return item_value == expected_value;
            }
        }
        return false;
    }

    false
}

#[derive(Clone, Default)]
pub struct InMemoryDynamoDb {
    store: Arc<Mutex<HashMap<String, TableStore>>>,
}

pub async fn create_in_memory_dynamodb_client() -> (Client, InMemoryDynamoDb) {
    let backend = InMemoryDynamoDb::new();
    let bound = crate::DynamoDbLocal::builder()
        .with_backend(backend.clone())
        .as_http_client();

    let client = bound.client().await;
    (client, backend)
}

impl InMemoryDynamoDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_table(&self, table_name: &str, key_schema: &[&str]) {
        match self.store.lock().unwrap().entry(table_name.to_string()) {
            Entry::Vacant(v) => {
                v.insert(TableStore {
                    schema: key_schema.iter().map(|s| s.to_string()).collect(),
                    items: HashMap::new(),
                });
            }
            Entry::Occupied(_) => {
                panic!("Table {table_name} already exists");
            }
        }
    }

    fn table(&self, table_name: &str) -> TableRef<'_> {
        TableRef {
            lock: self.store.lock().unwrap(),
            table_name: table_name.to_string(),
        }
    }
}

struct TableRef<'a> {
    lock: MutexGuard<'a, HashMap<String, TableStore>>,
    table_name: String,
}

impl<'a> TableRef<'a> {
    fn get_mut(&mut self) -> Option<&mut TableStore> {
        self.lock.get_mut(&self.table_name)
    }
}

struct TableStore {
    schema: Vec<String>,
    items: HashMap<Vec<String>, HashMap<String, dynamodb_local_server_sdk::model::AttributeValue>>,
}

impl TableStore {
    fn key_from_item(
        &self,
        item: &HashMap<String, dynamodb_local_server_sdk::model::AttributeValue>,
    ) -> Vec<String> {
        self.schema
            .iter()
            .map(|key| format!("{:?}", item.get(key).unwrap()))
            .collect()
    }
}

#[async_trait::async_trait]
impl DynamoDb for InMemoryDynamoDb {
    async fn get_item(
        &self,
        input: input::GetItemInput,
    ) -> Result<output::GetItemOutput, error::GetItemError> {
        let mut table = self.table(&input.table_name);

        let table_store = match table.get_mut() {
            Some(t) => t,
            None => {
                return Err(error::GetItemError::ResourceNotFoundException(
                    error::ResourceNotFoundException::builder()
                        .message(Some(format!("Table: {} not found", input.table_name)))
                        .build(),
                ));
            }
        };

        let key = table_store.key_from_item(&input.key);
        let item = table_store.items.get(&key).cloned();

        Ok(output::GetItemOutput {
            item,
            consumed_capacity: None,
        })
    }

    async fn put_item(
        &self,
        input: input::PutItemInput,
    ) -> Result<output::PutItemOutput, error::PutItemError> {
        let mut table = self.table(&input.table_name);

        let table_store = match table.get_mut() {
            Some(t) => t,
            None => {
                return Err(error::PutItemError::ResourceNotFoundException(
                    error::ResourceNotFoundException::builder()
                        .message(Some(format!("Table: {} not found", input.table_name)))
                        .build(),
                ));
            }
        };

        // Check condition expression if present
        if let Some(condition_expr) = &input.condition_expression {
            let key = table_store.key_from_item(&input.item);
            let existing_item = table_store.items.get(&key);

            let condition_met = evaluate_condition_expression(
                condition_expr,
                existing_item,
                input.expression_attribute_values.as_ref(),
            );

            if !condition_met {
                return Err(error::PutItemError::ConditionalCheckFailedException(
                    error::ConditionalCheckFailedException::builder()
                        .message(Some("The conditional request failed".to_string()))
                        .build(),
                ));
            }
        }

        let key = table_store.key_from_item(&input.item);
        table_store.items.insert(key, input.item);

        Ok(output::PutItemOutput {
            attributes: None,
            consumed_capacity: None,
            item_collection_metrics: None,
        })
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::types::AttributeValue;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_put_and_get_item() {
        tracing_subscriber::fmt::init();
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        // Put an item
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("test-id".to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        let _put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .send()
            .await
            .unwrap();

        // Get the item back
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("test-id".to_string()));

        let get_result = client
            .get_item()
            .table_name("test-table")
            .set_key(Some(key))
            .send()
            .await;

        assert!(get_result.is_ok());
        let response = get_result.unwrap();
        assert!(response.item.is_some());
    }

    #[tokio::test]
    async fn test_get_nonexistent_item() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        let mut key = HashMap::new();
        key.insert(
            "id".to_string(),
            AttributeValue::S("nonexistent".to_string()),
        );

        let get_result = client
            .get_item()
            .table_name("test-table")
            .set_key(Some(key))
            .send()
            .await;

        assert!(get_result.is_ok());
        let response = get_result.unwrap();
        assert!(response.item.is_none());
    }

    #[tokio::test]
    async fn test_conditional_put_attribute_not_exists_success() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("new-id".to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        let put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await;

        assert!(put_result.is_ok());
    }

    #[tokio::test]
    async fn test_conditional_put_attribute_not_exists_failure() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        // First, put an item
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            AttributeValue::S("existing-id".to_string()),
        );
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .send()
            .await
            .unwrap();

        // Try to put the same item with condition that it shouldn't exist
        // This should return a ConditionalCheckFailedException
        let put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await;

        assert!(put_result.is_err());
        match put_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::put_item::PutItemError::ConditionalCheckFailedException(_) => {
                // Expected error type
            }
            other => panic!("Expected ConditionalCheckFailedException, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_conditional_put_and_expression() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id", "sk"]);

        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("test-id".to_string()));
        item.insert("sk".to_string(), AttributeValue::S("test-sk".to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        let put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .condition_expression("attribute_not_exists(id) AND attribute_not_exists(sk)")
            .send()
            .await;

        assert!(put_result.is_ok());
    }

    #[tokio::test]
    async fn test_conditional_put_and_expression_partial_failure() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        // First, put an item with 'id'
        let mut existing_item = HashMap::new();
        existing_item.insert(
            "id".to_string(),
            AttributeValue::S("existing-id".to_string()),
        );
        existing_item.insert(
            "other".to_string(),
            AttributeValue::S("other-value".to_string()),
        );

        client
            .put_item()
            .table_name("test-table")
            .set_item(Some(existing_item))
            .send()
            .await
            .unwrap();

        // Now try to put an item with condition that both id AND sk should not exist
        // This should fail because 'id' exists (even though 'sk' doesn't exist)
        let mut new_item = HashMap::new();
        new_item.insert(
            "id".to_string(),
            AttributeValue::S("existing-id".to_string()),
        );
        new_item.insert("sk".to_string(), AttributeValue::S("new-sk".to_string()));

        let put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(new_item))
            .condition_expression("attribute_not_exists(id) AND attribute_not_exists(sk)")
            .send()
            .await;

        // This should fail because the AND condition requires BOTH to not exist
        assert!(put_result.is_err());
        match put_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::put_item::PutItemError::ConditionalCheckFailedException(_) => {
                // Expected error type
            }
            other => panic!("Expected ConditionalCheckFailedException, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_multiple_clients_same_store() {
        let (client1, store) = create_in_memory_dynamodb_client().await;
        store.create_table("shared-table", &["id"]);

        // Create second client by cloning the first
        let client2 = client1.clone();

        // Put item with client1
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("shared-id".to_string()));

        client1
            .put_item()
            .table_name("shared-table")
            .set_item(Some(item.clone()))
            .send()
            .await
            .unwrap();

        // Get item with client2
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("shared-id".to_string()));

        let get_result = client2
            .get_item()
            .table_name("shared-table")
            .set_key(Some(key))
            .send()
            .await;

        assert!(get_result.is_ok());
        let response = get_result.unwrap();
        assert!(response.item.is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn test_transact_write_items() {
        let (client, _store) = create_in_memory_dynamodb_client().await;

        let mut item1 = HashMap::new();
        item1.insert("id".to_string(), AttributeValue::S("txn-1".to_string()));
        item1.insert("data".to_string(), AttributeValue::S("value1".to_string()));

        let mut item2 = HashMap::new();
        item2.insert("id".to_string(), AttributeValue::S("txn-2".to_string()));
        item2.insert("data".to_string(), AttributeValue::S("value2".to_string()));

        let txn_result = client
            .transact_write_items()
            .transact_items(
                aws_sdk_dynamodb::types::TransactWriteItem::builder()
                    .put(
                        aws_sdk_dynamodb::types::Put::builder()
                            .table_name("test-table")
                            .set_item(Some(item1.clone()))
                            .build()
                            .unwrap(),
                    )
                    .build(),
            )
            .transact_items(
                aws_sdk_dynamodb::types::TransactWriteItem::builder()
                    .put(
                        aws_sdk_dynamodb::types::Put::builder()
                            .table_name("test-table")
                            .set_item(Some(item2.clone()))
                            .build()
                            .unwrap(),
                    )
                    .build(),
            )
            .send()
            .await;

        assert!(txn_result.is_ok());

        // Verify items were written
        let mut key1 = HashMap::new();
        key1.insert("id".to_string(), AttributeValue::S("txn-1".to_string()));
        key1.insert("data".to_string(), AttributeValue::S("value1".to_string()));

        let get_result = client
            .get_item()
            .table_name("test-table")
            .set_key(Some(key1))
            .send()
            .await;

        assert!(get_result.is_ok());
        assert!(get_result.unwrap().item.is_some());
    }

    #[tokio::test]
    async fn test_condition_expression_basic() {
        let (dynamodb_client, dynamodb_store) = create_in_memory_dynamodb_client().await;

        // Initialize the DynamoDB table for Slate
        dynamodb_store.create_table("test-table", &["shard_id", "sequence_number"]);

        // Create a test item
        let mut item = HashMap::new();
        item.insert(
            "shard_id".to_string(),
            AttributeValue::S("test-shard".to_string()),
        );
        item.insert(
            "sequence_number".to_string(),
            AttributeValue::S("1".to_string()),
        );
        item.insert(
            "batch_id".to_string(),
            AttributeValue::S("test-batch".to_string()),
        );

        // First put should succeed
        let result1 = dynamodb_client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .condition_expression("attribute_not_exists(sequence_number)")
            .send()
            .await;

        assert!(result1.is_ok(), "First put should succeed");

        // Second put with same key should fail
        let result2 = dynamodb_client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .condition_expression("attribute_not_exists(sequence_number)")
            .send()
            .await;

        assert!(result2.is_err(), "Second put should fail due to condition");

        // Verify it's the right type of error
        match result2.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::put_item::PutItemError::ConditionalCheckFailedException(_) => {
                // Expected error type
            }
            other => panic!("Expected ConditionalCheckFailedException, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_item_table_not_found() {
        let (client, _store) = create_in_memory_dynamodb_client().await;
        // Note: we don't insert the table, so it should not exist

        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("test-id".to_string()));

        let get_result = client
            .get_item()
            .table_name("nonexistent-table")
            .set_key(Some(key))
            .send()
            .await;

        assert!(get_result.is_err());
        match get_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException(e) => {
                assert!(
                    e.message()
                        .unwrap()
                        .contains("Table: nonexistent-table not found")
                );
            }
            other => panic!("Expected ResourceNotFoundException, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_put_item_table_not_found() {
        let (client, _store) = create_in_memory_dynamodb_client().await;
        // Note: we don't insert the table, so it should not exist

        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("test-id".to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        let put_result = client
            .put_item()
            .table_name("nonexistent-table")
            .set_item(Some(item))
            .send()
            .await;

        assert!(put_result.is_err());
        match put_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::put_item::PutItemError::ResourceNotFoundException(e) => {
                assert!(
                    e.message()
                        .unwrap()
                        .contains("Table: nonexistent-table not found")
                );
            }
            other => panic!("Expected ResourceNotFoundException, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "update_item not supported yet"]
    async fn test_update_item_table_not_found() {
        let (client, _store) = create_in_memory_dynamodb_client().await;
        // Note: we don't insert the table, so it should not exist

        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("test-id".to_string()));

        let update_result = client
            .update_item()
            .table_name("nonexistent-table")
            .set_key(Some(key))
            .update_expression("SET #name = :val")
            .expression_attribute_names("#name", "name")
            .expression_attribute_values(":val", AttributeValue::S("new-value".to_string()))
            .send()
            .await;

        assert!(update_result.is_err());
        match update_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::update_item::UpdateItemError::ResourceNotFoundException(e) => {
                assert!(e.message().unwrap().contains("Table: nonexistent-table not found"));
            }
            other => panic!("Expected ResourceNotFoundException, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "query not supported yet"]
    async fn test_query_table_not_found() {
        let (client, _store) = create_in_memory_dynamodb_client().await;
        // Note: we don't insert the table, so it should not exist

        let query_result = client
            .query()
            .table_name("nonexistent-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("test-key".to_string()))
            .send()
            .await;

        assert!(query_result.is_err());
        match query_result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::query::QueryError::ResourceNotFoundException(e) => {
                assert!(
                    e.message()
                        .unwrap()
                        .contains("Table: nonexistent-table not found")
                );
            }
            other => panic!("Expected ResourceNotFoundException, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_existing_functionality_still_works() {
        // This test ensures that existing functionality still works after our changes
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("test-table", &["id"]);

        // Put an item
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("test-id".to_string()));
        item.insert(
            "name".to_string(),
            AttributeValue::S("test-name".to_string()),
        );

        let put_result = client
            .put_item()
            .table_name("test-table")
            .set_item(Some(item.clone()))
            .send()
            .await;

        assert!(put_result.is_ok());

        // Get the item back
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("test-id".to_string()));

        let get_result = client
            .get_item()
            .table_name("test-table")
            .set_key(Some(key))
            .send()
            .await;

        assert!(get_result.is_ok());
        let response = get_result.unwrap();
        assert!(response.item.is_some());
        assert_eq!(
            response.item.unwrap().get("name").unwrap().as_s().unwrap(),
            "test-name"
        );
    }

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

        // Verify we can use the table
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("test-id".to_string()));

        let put_result = client
            .put_item()
            .table_name("new-table")
            .set_item(Some(item))
            .send()
            .await;

        assert!(put_result.is_ok());
    }

    #[tokio::test]
    async fn test_create_table_already_exists() {
        let (client, store) = create_in_memory_dynamodb_client().await;
        store.create_table("existing-table", &["id"]);

        let result = client
            .create_table()
            .table_name("existing-table")
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

        assert!(result.is_err());
        match result.unwrap_err().into_service_error() {
            aws_sdk_dynamodb::operation::create_table::CreateTableError::ResourceInUseException(
                _,
            ) => {}
            other => panic!("Expected ResourceInUseException, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "query not supported yet"]
    async fn test_query_operation() {
        let (client, store) = create_in_memory_dynamodb_client().await;

        // Test case (c): Query on completely empty table
        store.create_table("empty-table", &["pk"]);

        let empty_result = client
            .query()
            .table_name("empty-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("test-key".to_string()))
            .send()
            .await
            .unwrap();

        assert_eq!(empty_result.count(), 0);
        assert!(empty_result.items().is_empty());

        // Setup table with composite key for remaining tests
        store.create_table("test-table", &["pk", "sk"]);

        // Insert test items with different partition keys
        let items = vec![
            ("pk1", "sk1", "data1"),
            ("pk1", "sk2", "data2"),
            ("pk1", "sk3", "data3"),
            ("pk2", "sk1", "data4"),
            ("pk2", "sk2", "data5"),
            ("pk3", "sk1", "data6"),
        ];

        for (pk, sk, data) in &items {
            let mut item = HashMap::new();
            item.insert("pk".to_string(), AttributeValue::S(pk.to_string()));
            item.insert("sk".to_string(), AttributeValue::S(sk.to_string()));
            item.insert("data".to_string(), AttributeValue::S(data.to_string()));

            client
                .put_item()
                .table_name("test-table")
                .set_item(Some(item))
                .send()
                .await
                .unwrap();
        }

        // Test case (a): Query includes items with matching hash key
        let pk1_result = client
            .query()
            .table_name("test-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("pk1".to_string()))
            .send()
            .await
            .unwrap();

        assert_eq!(pk1_result.count(), 3);
        let pk1_items = pk1_result.items();
        assert_eq!(pk1_items.len(), 3);

        // Verify all returned items have pk1
        for item in pk1_items {
            assert_eq!(item.get("pk").unwrap().as_s().unwrap(), "pk1");
        }

        // Verify sort key ordering (should be sk1, sk2, sk3)
        let sort_keys: Vec<_> = pk1_items
            .iter()
            .map(|item| item.get("sk").unwrap().as_s().unwrap())
            .collect();
        assert_eq!(sort_keys, vec!["sk1", "sk2", "sk3"]);

        // Test case (b): Query omits items with different hash keys
        let pk2_result = client
            .query()
            .table_name("test-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("pk2".to_string()))
            .send()
            .await
            .unwrap();

        assert_eq!(pk2_result.count(), 2);
        let pk2_items = pk2_result.items();

        // Verify no pk1 or pk3 items are returned
        for item in pk2_items {
            assert_eq!(item.get("pk").unwrap().as_s().unwrap(), "pk2");
            assert_ne!(item.get("pk").unwrap().as_s().unwrap(), "pk1");
            assert_ne!(item.get("pk").unwrap().as_s().unwrap(), "pk3");
        }

        // Test case (c): Query returns empty for non-existent hash key in non-empty table
        let nonexistent_result = client
            .query()
            .table_name("test-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("nonexistent".to_string()))
            .send()
            .await
            .unwrap();

        assert_eq!(nonexistent_result.count(), 0);
        assert!(nonexistent_result.items().is_empty());

        // Test reverse sort order
        let pk1_reverse = client
            .query()
            .table_name("test-table")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S("pk1".to_string()))
            .scan_index_forward(false)
            .send()
            .await
            .unwrap();

        let reverse_sort_keys: Vec<_> = pk1_reverse
            .items()
            .iter()
            .map(|item| item.get("sk").unwrap().as_s().unwrap())
            .collect();
        assert_eq!(reverse_sort_keys, vec!["sk3", "sk2", "sk1"]);
    }
}
