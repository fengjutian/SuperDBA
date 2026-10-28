//! Parity tests for multi-database operations — using the secondary database
//! (superdba_test_secondary) with its `secondary_schema.remote_data` table.

use serde_json::Value;

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_get_schemas_secondary() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity_secondary("get_schemas:secondary", |driver, params| async move {
            driver.get_schemas(&params).await
        })
        .await;

    let schemas: Vec<String> = serde_json::from_value(result).unwrap();
    assert!(
        schemas.contains(&"secondary_schema".to_string()),
        "secondary database should contain secondary_schema, got: {:?}",
        schemas
    );
}

#[tokio::test]
#[ignore]
async fn parity_get_columns_secondary() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity_secondary("get_columns:remote_data", |driver, params| async move {
            driver
                .get_columns(&params, "remote_data", Some("secondary_schema"))
                .await
        })
        .await;

    let columns = result.as_array().expect("columns should be an array");
    assert!(!columns.is_empty(), "remote_data table should have columns");

    let col_names: Vec<&str> = columns
        .iter()
        .filter_map(|c| c.get("name").and_then(Value::as_str))
        .collect();
    assert!(
        col_names.contains(&"id"),
        "remote_data should have an id column, got: {:?}",
        col_names
    );
    assert!(
        col_names.contains(&"value"),
        "remote_data should have a value column, got: {:?}",
        col_names
    );
}

#[tokio::test]
#[ignore]
async fn parity_execute_query_secondary() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity_secondary("execute_query:secondary", |driver, params| async move {
            driver
                .execute_query(
                    &params,
                    "SELECT id, value FROM secondary_schema.remote_data ORDER BY id LIMIT 5",
                    Some(100),
                    1,
                    Some("secondary_schema"),
                )
                .await
        })
        .await;

    let rows = result.get("rows").and_then(Value::as_array).unwrap();
    assert!(!rows.is_empty(), "secondary query should return rows");
}
