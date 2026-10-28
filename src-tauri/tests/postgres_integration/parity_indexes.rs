//! Parity tests for index introspection — covers baseline tests from
//! `indexes.rs` that are NOT already covered in `parity_tests.rs`.
//!
//! Already covered by `parity_tests.rs`:
//!   - parity_get_indexes (basic: all_types table, checks non-empty)
//!
//! New in this file — each test calls `get_indexes` through the trait via the
//! harness and uses `assert_parity()` for byte-perfect JSON comparison.

use serde_json::Value;

use crate::parity::ParityHarness;

/// Parity equivalent of `test_get_indexes_btree`.
/// Verifies a specific btree index is present with correct attributes.
#[tokio::test]
#[ignore]
async fn parity_get_indexes_btree() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_indexes:all_types:btree", |driver, params| async move {
            driver
                .get_indexes(&params, "all_types", Some("test_schema"))
                .await
        })
        .await;

    let arr = result.as_array().expect("indexes should be an array");
    let idx = arr
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("idx_all_types_text"))
        .expect("Expected idx_all_types_text index");

    assert_eq!(
        idx.get("column_name").and_then(|v| v.as_str()),
        Some("col_text"),
        "idx_all_types_text should be on col_text"
    );
    assert_eq!(
        idx.get("is_unique").and_then(|v| v.as_bool()),
        Some(false),
        "idx_all_types_text should not be unique"
    );
    assert_eq!(
        idx.get("is_primary").and_then(|v| v.as_bool()),
        Some(false),
        "idx_all_types_text should not be primary"
    );
}

/// Parity equivalent of `test_get_indexes_unique`.
/// Verifies a unique index is reported with correct flags.
#[tokio::test]
#[ignore]
async fn parity_get_indexes_unique() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_indexes:all_types:unique",
            |driver, params| async move {
                driver
                    .get_indexes(&params, "all_types", Some("test_schema"))
                    .await
            },
        )
        .await;

    let arr = result.as_array().expect("indexes should be an array");
    let idx = arr
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("idx_all_types_uuid"))
        .expect("Expected idx_all_types_uuid unique index");

    assert_eq!(
        idx.get("column_name").and_then(|v| v.as_str()),
        Some("col_uuid"),
        "idx_all_types_uuid should be on col_uuid"
    );
    assert_eq!(
        idx.get("is_unique").and_then(|v| v.as_bool()),
        Some(true),
        "idx_all_types_uuid should be unique"
    );
}

/// Parity equivalent of `test_get_indexes_composite`.
/// Verifies a composite (multi-column) index is reported with correct seq_in_index.
#[tokio::test]
#[ignore]
async fn parity_get_indexes_composite() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_indexes:order_items:composite",
            |driver, params| async move {
                driver
                    .get_indexes(&params, "order_items", Some("test_schema"))
                    .await
            },
        )
        .await;

    let arr = result.as_array().expect("indexes should be an array");

    // The composite index idx_order_items_composite covers (order_id, product)
    let idx_entries: Vec<&Value> = arr
        .iter()
        .filter(|i| i.get("name").and_then(|n| n.as_str()) == Some("idx_order_items_composite"))
        .collect();

    assert_eq!(
        idx_entries.len(),
        2,
        "Composite index should have 2 entries (one per column)"
    );

    // Verify seq_in_index ordering
    let first = idx_entries
        .iter()
        .find(|i| i.get("seq_in_index").and_then(|v| v.as_u64()) == Some(1))
        .expect("should have entry with seq_in_index=1");
    assert_eq!(
        first.get("column_name").and_then(|v| v.as_str()),
        Some("order_id")
    );

    let second = idx_entries
        .iter()
        .find(|i| i.get("seq_in_index").and_then(|v| v.as_u64()) == Some(2))
        .expect("should have entry with seq_in_index=2");
    assert_eq!(
        second.get("column_name").and_then(|v| v.as_str()),
        Some("product")
    );
}

/// Parity equivalent of `test_get_indexes_primary_key`.
/// Verifies the primary key index is correctly reported.
#[tokio::test]
#[ignore]
async fn parity_get_indexes_primary_key() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_indexes:all_types:primary_key",
            |driver, params| async move {
                driver
                    .get_indexes(&params, "all_types", Some("test_schema"))
                    .await
            },
        )
        .await;

    let arr = result.as_array().expect("indexes should be an array");
    let pk = arr
        .iter()
        .find(|i| i.get("is_primary").and_then(|v| v.as_bool()) == Some(true))
        .expect("Expected primary key index");

    assert_eq!(
        pk.get("column_name").and_then(|v| v.as_str()),
        Some("id"),
        "PK should be on id column"
    );
    assert_eq!(
        pk.get("is_unique").and_then(|v| v.as_bool()),
        Some(true),
        "PK index should also be unique"
    );
}
