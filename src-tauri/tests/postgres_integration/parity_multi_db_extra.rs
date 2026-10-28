//! Extra parity tests for multi-database operations — get_databases_lists_both,
//! get_tables_secondary, pool_isolation, fallback_to_maintenance_db.

use serde_json::Value;
use superdba_lib::models::DatabaseSelection;

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_get_databases_lists_both() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_databases:lists_both", |driver, params| async move {
            driver.get_databases(&params).await
        })
        .await;

    let databases: Vec<String> = serde_json::from_value(result).unwrap();
    assert!(
        databases.contains(&"testdb".to_string()),
        "Should list testdb, got: {:?}",
        databases
    );
    assert!(
        databases.contains(&"superdba_test_secondary".to_string()),
        "Should list superdba_test_secondary, got: {:?}",
        databases
    );
}

#[tokio::test]
#[ignore]
async fn parity_get_tables_secondary_schema() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity_secondary("get_tables:secondary_extra", |driver, params| async move {
            driver.get_tables(&params, Some("secondary_schema")).await
        })
        .await;

    let tables = result.as_array().expect("tables should be an array");
    let table_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str))
        .collect();
    assert!(
        table_names.contains(&"remote_data"),
        "Expected remote_data table in secondary, got: {:?}",
        table_names
    );
}

#[tokio::test]
#[ignore]
async fn parity_pool_isolation_between_databases() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // Primary database should see test_schema tables
    let primary_result = harness
        .assert_parity(
            "get_tables:primary_pool_isolation",
            |driver, params| async move { driver.get_tables(&params, Some("test_schema")).await },
        )
        .await;

    let primary_tables = primary_result.as_array().expect("tables should be array");
    assert!(
        !primary_tables.is_empty(),
        "Primary db should have test_schema tables"
    );

    // Secondary database should NOT have test_schema
    let secondary_result = harness
        .assert_parity_secondary(
            "get_schemas:pool_isolation_secondary",
            |driver, params| async move { driver.get_schemas(&params).await },
        )
        .await;

    let secondary_schemas: Vec<String> = serde_json::from_value(secondary_result).unwrap();
    assert!(
        !secondary_schemas.contains(&"test_schema".to_string()),
        "test_schema should not exist in secondary database, got: {:?}",
        secondary_schemas
    );
}

#[tokio::test]
#[ignore]
async fn parity_fallback_to_maintenance_db() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // Connect with "postgres" maintenance database — should be able to list databases
    let result = harness
        .assert_parity(
            "get_databases:maintenance_db",
            |driver, params| async move {
                let mut maint_params = params.clone();
                maint_params.database = DatabaseSelection::Single("postgres".to_string());
                driver.get_databases(&maint_params).await
            },
        )
        .await;

    let databases: Vec<String> = serde_json::from_value(result).unwrap();
    assert!(
        databases.contains(&"testdb".to_string()),
        "Maintenance db should list testdb, got: {:?}",
        databases
    );
}
