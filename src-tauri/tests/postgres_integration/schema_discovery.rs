//! Schema discovery tests: get_schemas, get_databases, get_tables.

use crate::helpers::pg_params;
use superdba_lib::drivers::postgres;

#[tokio::test]
#[ignore]
async fn test_get_schemas_returns_test_schema() {
    require_pg!();
    let params = pg_params();

    let schemas = postgres::get_schemas(&params)
        .await
        .expect("get_schemas should succeed");

    assert!(
        schemas.contains(&"test_schema".to_string()),
        "Expected test_schema in schemas list, got: {:?}",
        schemas
    );
    assert!(
        schemas.contains(&"other_schema".to_string()),
        "Expected other_schema in schemas list"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_databases_returns_testdb() {
    require_pg!();
    let params = pg_params();

    let databases = postgres::get_databases(&params)
        .await
        .expect("get_databases should succeed");

    assert!(
        databases.contains(&"testdb".to_string()),
        "Expected testdb in databases list, got: {:?}",
        databases
    );
    assert!(
        databases.contains(&"superdba_test_secondary".to_string()),
        "Expected superdba_test_secondary in databases list"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_tables_returns_seeded_tables() {
    require_pg!();
    let params = pg_params();

    let tables = postgres::get_tables(&params, "test_schema")
        .await
        .expect("get_tables should succeed");

    let table_names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();

    assert!(
        table_names.contains(&"all_types"),
        "Expected all_types table"
    );
    assert!(
        table_names.contains(&"with_enum"),
        "Expected with_enum table"
    );
    assert!(table_names.contains(&"orders"), "Expected orders table");
    assert!(
        table_names.contains(&"order_items"),
        "Expected order_items table"
    );
    assert!(
        table_names.contains(&"with_cross_schema_fk"),
        "Expected with_cross_schema_fk table"
    );
    assert!(
        table_names.contains(&"crud_scratch"),
        "Expected crud_scratch table"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_tables_other_schema() {
    require_pg!();
    let params = pg_params();

    let tables = postgres::get_tables(&params, "other_schema")
        .await
        .expect("get_tables for other_schema should succeed");

    let table_names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(
        table_names.contains(&"lookup"),
        "Expected lookup table in other_schema"
    );
}
