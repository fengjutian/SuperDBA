//! Extra parity tests for query execution — affected_rows_for_dml, batch_session_state.

use serde_json::Value;

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_execute_query_affected_rows_for_dml() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // Insert into scratch table — DML should report affected_rows = 1
    // and return no columns/rows.
    let result = harness
        .assert_parity(
            "execute_query:affected_rows_dml",
            |driver, params| async move {
                driver
                    .execute_query(
                        &params,
                        "INSERT INTO test_schema.crud_scratch (name, value) \
                         VALUES ('parity_affected_rows', 1)",
                        None,
                        1,
                        Some("test_schema"),
                    )
                    .await
            },
        )
        .await;

    // DML returns affected_rows = 1
    let affected = result
        .get("affected_rows")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    assert_eq!(affected, 1, "INSERT should affect exactly 1 row");

    // DML returns no columns
    let columns = result
        .get("columns")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(columns, 0, "DML should return no columns");

    // DML returns no rows
    let rows = result
        .get("rows")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(rows, 0, "DML should return no rows");
}

#[tokio::test]
#[ignore]
async fn parity_execute_batch_session_state() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // execute_batch returns Vec<BatchStatementResult>, and each entry
    // carries execution_time_ms: Option<f64> — a genuine wall-clock value
    // that can never byte-match between two separate driver processes.
    // assert_parity's exact comparison is the wrong tool here (same class
    // of issue fixed in parity_batch.rs) — call each target directly and
    // check only the deterministic fields.
    let statements = vec![
        "BEGIN".to_string(),
        "CREATE TEMP TABLE _parity_batch_test (x INT)".to_string(),
        "INSERT INTO _parity_batch_test VALUES (42)".to_string(),
        "SELECT x FROM _parity_batch_test".to_string(),
        "COMMIT".to_string(),
    ];

    for (target, driver) in harness.targets() {
        let result = driver
            .execute_batch(
                &harness.params,
                &statements,
                Some(100),
                1,
                Some("test_schema"),
                None,
            )
            .await
            .unwrap_or_else(|e| panic!("execute_batch failed on {}: {}", target, e));
        let arr = serde_json::to_value(&result).expect("serialize batch result");
        let arr = arr.as_array().expect("batch result should be an array");

        assert!(
            arr.len() >= 4,
            "{}: expected at least 4 results, got: {}",
            target,
            arr.len()
        );

        // The SELECT result (4th statement, index 3) should return the inserted value
        let select_result = &arr[3];
        let succeeded = select_result
            .get("error")
            .map(Value::is_null)
            .unwrap_or(false);
        assert!(
            succeeded,
            "{}: SELECT from temp table should succeed, got: {:?}",
            target, select_result
        );

        if let Some(result_obj) = select_result.get("result") {
            if let Some(rows) = result_obj.get("rows").and_then(Value::as_array) {
                assert_eq!(rows.len(), 1, "{}: SELECT should return 1 row", target);
                if let Some(row) = rows.first().and_then(Value::as_array) {
                    assert_eq!(
                        row.first().and_then(Value::as_i64),
                        Some(42),
                        "{}: temp table should contain value 42",
                        target
                    );
                }
            }
        }
    }
}
