//! Parity tests for schema discovery — covers baseline tests from
//! `schema_discovery.rs` that are NOT already covered in `parity_tests.rs`.
//!
//! Already covered by `parity_tests.rs`:
//!   - parity_get_schemas (covers test_get_schemas_returns_test_schema)
//!   - parity_get_databases (covers test_get_databases_returns_testdb)
//!   - parity_get_tables (covers test_get_tables_returns_seeded_tables)
//!
//! New in this file:
//!   - parity_get_tables_other_schema (covers test_get_tables_other_schema)

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_get_tables_other_schema() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_tables:other_schema", |driver, params| async move {
            driver.get_tables(&params, Some("other_schema")).await
        })
        .await;

    let arr = result.as_array().expect("tables should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|t| t.get("name")?.as_str()).collect();
    assert!(
        names.contains(&"lookup"),
        "Expected lookup table in other_schema, got: {:?}",
        names
    );
}
