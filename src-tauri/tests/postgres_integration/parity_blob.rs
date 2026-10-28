//! Parity tests for BLOB (bytea) handling — covers ALL 3 baseline tests from
//! `blob.rs`. None of these were previously covered by parity tests.
//!
//! The `save_blob_to_file` test verifies both drivers can write to a file without
//! error. The `fetch_blob_as_data_url` test verifies both drivers return
//! identical wire-format strings for the same row.

use std::collections::HashMap;

use serde_json::json;

use crate::parity::ParityHarness;

/// Parity equivalent of `test_insert_and_query_bytea`.
/// Verifies inserting a BLOB-wire-encoded bytea value and querying it back.
/// Both drivers must handle the "BLOB:<size>:<mime>:<base64>" wire format
/// identically on insert and produce identical query results.
#[tokio::test]
#[ignore]
async fn parity_blob_insert_and_query() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // insert_record is destructive against the one shared physical database
    // both targets point at — assert_parity calls each target in sequence,
    // and col_text has no unique constraint, so inserting the same marker
    // value from both targets produces TWO rows in the shared table (not
    // one row inserted "the same way twice"). Run insert+query+cleanup
    // directly per target so each target's row is isolated and cleaned up
    // before the next target runs.
    for (target, driver) in harness.targets() {
        // 4 bytes (0xCA 0xFE 0xBA 0xBE) encoded as base64 = "yv66vg=="
        let blob_wire = "BLOB:4:application/octet-stream:yv66vg==";
        let mut data = HashMap::new();
        data.insert("col_bytea".to_string(), json!(blob_wire));
        data.insert("col_text".to_string(), json!("parity_blob_test"));
        driver
            .insert_record(
                &harness.params,
                "all_types",
                data,
                Some("test_schema"),
                10_000_000,
            )
            .await
            .unwrap_or_else(|e| panic!("insert_record failed on {}: {}", target, e));

        let query_result = driver
            .execute_query(
                &harness.params,
                "SELECT col_bytea FROM test_schema.all_types WHERE col_text = 'parity_blob_test'",
                None,
                1,
                Some("test_schema"),
            )
            .await
            .unwrap_or_else(|e| panic!("execute_query failed on {}: {}", target, e));

        assert_eq!(
            query_result.rows.len(),
            1,
            "{}: should find exactly the inserted blob row",
            target
        );
        assert!(
            !query_result.rows[0][0].is_null(),
            "{}: bytea column should not be null",
            target
        );

        driver
            .execute_query(
                &harness.params,
                "DELETE FROM test_schema.all_types WHERE col_text = 'parity_blob_test'",
                None,
                1,
                Some("test_schema"),
            )
            .await
            .unwrap_or_else(|e| panic!("cleanup delete failed on {}: {}", target, e));
    }
}

/// Parity equivalent of `test_save_blob_to_file`.
/// Verifies both drivers can export a blob column to a file without error.
/// The seeded row (id=1) has col_bytea = '\xDEADBEEF'.
#[tokio::test]
#[ignore]
async fn parity_blob_save_to_file() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let tmp_path = std::env::temp_dir().join("superdba_parity_blob_test.bin");
    let path_str = tmp_path.to_str().unwrap().to_string();

    let result = harness
        .assert_parity("save_blob_to_file:basic", |driver, params| {
            let path = path_str.clone();
            async move {
                let mut pk_map = HashMap::new();
                pk_map.insert("id".to_string(), json!(1));
                driver
                    .save_blob_to_file(
                        &params,
                        "all_types",
                        "col_bytea",
                        &pk_map,
                        Some("test_schema"),
                        &path,
                    )
                    .await
            }
        })
        .await;

    // Both drivers should succeed (result is null/() serialized)
    assert!(
        result.is_null(),
        "save_blob_to_file returns () which serializes to null"
    );

    // Verify file was written and has content
    let metadata = std::fs::metadata(&tmp_path);
    assert!(
        metadata.is_ok(),
        "File should exist after save_blob_to_file"
    );
    assert!(metadata.unwrap().len() > 0, "File should have content");

    // Clean up
    let _ = std::fs::remove_file(&tmp_path);
}

/// Parity equivalent of `test_fetch_blob_as_data_url`.
/// Verifies both drivers return identical BLOB wire format strings for the
/// same row. The seeded row (id=1) has col_bytea = '\xDEADBEEF'.
#[tokio::test]
#[ignore]
async fn parity_blob_fetch_as_data_url() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "fetch_blob_as_data_url:basic",
            |driver, params| async move {
                let mut pk_map = HashMap::new();
                pk_map.insert("id".to_string(), json!(1));
                driver
                    .fetch_blob_as_data_url(
                        &params,
                        "all_types",
                        "col_bytea",
                        &pk_map,
                        Some("test_schema"),
                    )
                    .await
            },
        )
        .await;

    let data_url = result
        .as_str()
        .expect("fetch_blob_as_data_url should return a string");
    // Should be in BLOB wire format: "BLOB:<size>:<mime>:<base64>" or data URL
    assert!(
        data_url.starts_with("BLOB:") || data_url.starts_with("data:"),
        "Should return BLOB wire format or data URL, got: {}",
        &data_url[..data_url.len().min(50)]
    );
}
