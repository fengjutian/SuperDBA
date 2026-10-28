//! CRUD operation tests (insert, update, delete).

use crate::helpers::pg_params;
use serde_json::json;
use std::collections::HashMap;
use superdba_lib::drivers::postgres;

#[tokio::test]
#[ignore]
async fn test_insert_basic_types() {
    require_pg!();
    let params = pg_params();

    let mut data = HashMap::new();
    data.insert("name".to_string(), json!("insert_test"));
    data.insert("value".to_string(), json!(42));

    let affected =
        postgres::insert_record(&params, "crud_scratch", data, "test_schema", 10_000_000)
            .await
            .expect("insert_record should succeed");

    assert_eq!(affected, 1);

    // Cleanup
    let _ = postgres::execute_query(
        &params,
        "DELETE FROM test_schema.crud_scratch WHERE name = 'insert_test'",
        None,
        1,
        None,
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_insert_null_values() {
    require_pg!();
    let params = pg_params();

    let mut data = HashMap::new();
    data.insert("name".to_string(), json!(null));
    data.insert("value".to_string(), json!(null));

    let affected =
        postgres::insert_record(&params, "crud_scratch", data, "test_schema", 10_000_000)
            .await
            .expect("insert_record with nulls should succeed");

    assert_eq!(affected, 1);

    // Cleanup — delete rows with null name (our insert)
    let _ = postgres::execute_query(
        &params,
        "DELETE FROM test_schema.crud_scratch WHERE name IS NULL",
        None,
        1,
        None,
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_update_with_single_pk() {
    require_pg!();
    let params = pg_params();

    // Insert a row to update
    let mut insert_data = HashMap::new();
    insert_data.insert("name".to_string(), json!("to_update"));
    insert_data.insert("value".to_string(), json!(1));
    postgres::insert_record(
        &params,
        "crud_scratch",
        insert_data,
        "test_schema",
        10_000_000,
    )
    .await
    .expect("insert for update test");

    // Find the row's ID
    let result = postgres::execute_query(
        &params,
        "SELECT id FROM test_schema.crud_scratch WHERE name = 'to_update' ORDER BY id DESC LIMIT 1",
        None,
        1,
        None,
    )
    .await
    .expect("find row");
    let row_id = result.rows[0][0].as_i64().unwrap();

    // Update it (update_record updates one column at a time)
    let mut pk_map = HashMap::new();
    pk_map.insert("id".to_string(), json!(row_id));

    let affected = postgres::update_record(
        &params,
        "crud_scratch",
        &pk_map,
        "value",
        json!(999),
        "test_schema",
        10_000_000,
    )
    .await
    .expect("update_record should succeed");

    assert_eq!(affected, 1);

    // Verify the update took effect
    let verify = postgres::execute_query(
        &params,
        &format!(
            "SELECT value FROM test_schema.crud_scratch WHERE id = {}",
            row_id
        ),
        None,
        1,
        None,
    )
    .await
    .unwrap();
    assert_eq!(verify.rows[0][0], json!(999));
}

#[tokio::test]
#[ignore]
async fn test_update_composite_pk() {
    require_pg!();
    let params = pg_params();

    // order_items has composite PK (order_id, item_no)
    let mut pk_map = HashMap::new();
    pk_map.insert("order_id".to_string(), json!(1));
    pk_map.insert("item_no".to_string(), json!(1));

    let affected = postgres::update_record(
        &params,
        "order_items",
        &pk_map,
        "product",
        json!("Updated Widget"),
        "test_schema",
        10_000_000,
    )
    .await
    .expect("update_record with composite PK should succeed");

    assert_eq!(affected, 1);

    // Restore original value
    let _ = postgres::update_record(
        &params,
        "order_items",
        &pk_map,
        "product",
        json!("Widget"),
        "test_schema",
        10_000_000,
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_delete_single_pk() {
    require_pg!();
    let params = pg_params();

    // Insert a row to delete
    let mut data = HashMap::new();
    data.insert("name".to_string(), json!("to_delete"));
    data.insert("value".to_string(), json!(0));
    postgres::insert_record(&params, "crud_scratch", data, "test_schema", 10_000_000)
        .await
        .expect("insert for delete test");

    // Find the row
    let result = postgres::execute_query(
        &params,
        "SELECT id FROM test_schema.crud_scratch WHERE name = 'to_delete' ORDER BY id DESC LIMIT 1",
        None,
        1,
        None,
    )
    .await
    .unwrap();
    let row_id = result.rows[0][0].as_i64().unwrap();

    // Delete it
    let mut pk_map = HashMap::new();
    pk_map.insert("id".to_string(), json!(row_id));

    let affected = postgres::delete_record(&params, "crud_scratch", &pk_map, "test_schema")
        .await
        .expect("delete_record should succeed");

    assert_eq!(affected, 1);

    // Verify gone
    let verify = postgres::execute_query(
        &params,
        &format!(
            "SELECT COUNT(*) FROM test_schema.crud_scratch WHERE id = {}",
            row_id
        ),
        None,
        1,
        None,
    )
    .await
    .unwrap();
    assert_eq!(verify.rows[0][0], json!(0_i64));
}

#[tokio::test]
#[ignore]
async fn test_insert_json_object() {
    require_pg!();
    let params = pg_params();

    let mut data = HashMap::new();
    data.insert(
        "col_jsonb".to_string(),
        json!({"nested": {"key": "value"}, "arr": [1, 2, 3]}),
    );
    data.insert("col_text".to_string(), json!("json_test"));

    let affected = postgres::insert_record(&params, "all_types", data, "test_schema", 10_000_000)
        .await
        .expect("insert JSON object should succeed");

    assert_eq!(affected, 1);

    // Clean up
    let _ = postgres::execute_query(
        &params,
        "DELETE FROM test_schema.all_types WHERE col_text = 'json_test'",
        None,
        1,
        None,
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_insert_array_value() {
    require_pg!();
    let params = pg_params();

    let mut data = HashMap::new();
    data.insert("col_int_array".to_string(), json!([10, 20, 30]));
    data.insert("col_text".to_string(), json!("array_test"));

    let affected = postgres::insert_record(&params, "all_types", data, "test_schema", 10_000_000)
        .await
        .expect("insert array value should succeed");

    assert_eq!(affected, 1);

    // Verify round-trip
    let result = postgres::execute_query(
        &params,
        "SELECT col_int_array FROM test_schema.all_types WHERE col_text = 'array_test'",
        None,
        1,
        None,
    )
    .await
    .unwrap();

    assert_eq!(result.rows.len(), 1);
    assert!(
        result.rows[0][0].is_array(),
        "Expected array, got: {:?}",
        result.rows[0][0]
    );

    // Clean up
    let _ = postgres::execute_query(
        &params,
        "DELETE FROM test_schema.all_types WHERE col_text = 'array_test'",
        None,
        1,
        None,
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_insert_enum_value() {
    require_pg!();
    let params = pg_params();

    let mut data = HashMap::new();
    data.insert("current_mood".to_string(), json!("sad"));

    let affected = postgres::insert_record(&params, "with_enum", data, "test_schema", 10_000_000)
        .await
        .expect("insert enum value should succeed");

    assert_eq!(affected, 1);

    // Clean up the extra row
    let _ = postgres::execute_query(
        &params,
        "DELETE FROM test_schema.with_enum WHERE id > 1",
        None,
        1,
        None,
    )
    .await;
}
