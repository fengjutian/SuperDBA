//! Index introspection tests.

use crate::helpers::pg_params;
use superdba_lib::drivers::postgres;

#[tokio::test]
#[ignore]
async fn test_get_indexes_btree() {
    require_pg!();
    let params = pg_params();

    let indexes = postgres::get_indexes(&params, "all_types", "test_schema")
        .await
        .expect("get_indexes should succeed");

    let idx = indexes.iter().find(|i| i.name == "idx_all_types_text");
    assert!(idx.is_some(), "Expected idx_all_types_text index");
    let idx = idx.unwrap();
    assert_eq!(idx.column_name, "col_text");
    assert!(!idx.is_unique);
    assert!(!idx.is_primary);
}

#[tokio::test]
#[ignore]
async fn test_get_indexes_unique() {
    require_pg!();
    let params = pg_params();

    let indexes = postgres::get_indexes(&params, "all_types", "test_schema")
        .await
        .expect("get_indexes should succeed");

    let idx = indexes.iter().find(|i| i.name == "idx_all_types_uuid");
    assert!(idx.is_some(), "Expected idx_all_types_uuid unique index");
    let idx = idx.unwrap();
    assert_eq!(idx.column_name, "col_uuid");
    assert!(idx.is_unique);
}

#[tokio::test]
#[ignore]
async fn test_get_indexes_composite() {
    require_pg!();
    let params = pg_params();

    let indexes = postgres::get_indexes(&params, "order_items", "test_schema")
        .await
        .expect("get_indexes should succeed");

    // The composite index idx_order_items_composite covers (order_id, product)
    let idx_entries: Vec<_> = indexes
        .iter()
        .filter(|i| i.name == "idx_order_items_composite")
        .collect();

    assert_eq!(
        idx_entries.len(),
        2,
        "Composite index should have 2 entries (one per column)"
    );
    // Verify seq_in_index ordering
    let first = idx_entries.iter().find(|i| i.seq_in_index == 1).unwrap();
    assert_eq!(first.column_name, "order_id");
    let second = idx_entries.iter().find(|i| i.seq_in_index == 2).unwrap();
    assert_eq!(second.column_name, "product");
}

#[tokio::test]
#[ignore]
async fn test_get_indexes_primary_key() {
    require_pg!();
    let params = pg_params();

    let indexes = postgres::get_indexes(&params, "all_types", "test_schema")
        .await
        .expect("get_indexes should succeed");

    let pk = indexes.iter().find(|i| i.is_primary);
    assert!(pk.is_some(), "Expected primary key index");
    let pk = pk.unwrap();
    assert_eq!(pk.column_name, "id");
    assert!(pk.is_unique, "PK index should also be unique");
}
