//! Parity tests for `execute_batch` — ensures plugin handles multi-statement
//! batch execution identically to the built-in driver.
//!
//! `execute_batch` returns `Vec<BatchStatementResult>`, and each entry
//! carries `execution_time_ms: Option<f64>` — a genuinely non-deterministic
//! wall-clock value that can never byte-match between two separate driver
//! processes. `assert_parity`'s exact JSON comparison is the wrong tool here
//! (same class of issue as `explain_query`'s volatile cost/timing output in
//! `parity_explain.rs`). Instead, call each target directly and compare only
//! the deterministic fields (`error`, `result`), ignoring `execution_time_ms`.

use serde_json::Value;

use crate::parity::ParityHarness;

/// Strip the non-deterministic `execution_time_ms` field from each batch
/// entry so the remaining structure (`error`, `result`) can be compared
/// exactly across targets.
fn normalize_batch_result(v: &Value) -> Value {
    let arr = v.as_array().expect("batch result should be an array");
    Value::Array(
        arr.iter()
            .map(|entry| json_without_key(entry, "execution_time_ms"))
            .collect(),
    )
}

fn json_without_key(v: &Value, key: &str) -> Value {
    match v.as_object() {
        Some(obj) => {
            let mut filtered = serde_json::Map::new();
            for (k, val) in obj {
                if k != key {
                    filtered.insert(k.clone(), val.clone());
                }
            }
            Value::Object(filtered)
        }
        None => v.clone(),
    }
}

#[tokio::test]
#[ignore]
async fn parity_batch_session_state() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let queries = vec![
        "SET search_path TO test_schema".to_string(),
        "SELECT current_schema() AS current_schema".to_string(),
    ];

    let mut normalized_results = Vec::new();
    for (target, driver) in harness.targets() {
        let result = driver
            .execute_batch(
                &harness.params,
                &queries,
                Some(100),
                1,
                Some("test_schema"),
                None,
            )
            .await
            .unwrap_or_else(|e| panic!("execute_batch failed on {}: {}", target, e));
        let json = serde_json::to_value(&result).expect("serialize batch result");
        normalized_results.push((target.to_string(), normalize_batch_result(&json)));
    }

    for window in normalized_results.windows(2) {
        assert_eq!(
            window[0].1, window[1].1,
            "execute_batch:session_state parity failure between {} and {}",
            window[0].0, window[1].0
        );
    }

    let arr = normalized_results[0].1.as_array().unwrap();
    assert_eq!(arr.len(), 2, "should have results for both statements");
    let second = &arr[1];
    let succeeded = second.get("error").map(Value::is_null).unwrap_or(false);
    assert!(
        succeeded,
        "SELECT current_schema() should succeed, got: {:?}",
        second
    );
}

#[tokio::test]
#[ignore]
async fn parity_batch_mixed_statements() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let queries = vec![
        "SELECT id FROM test_schema.all_types ORDER BY id LIMIT 2".to_string(),
        "INSERT INTO test_schema.crud_scratch(name, value) VALUES ('batch_parity', 42)".to_string(),
    ];

    let mut normalized_results = Vec::new();
    for (target, driver) in harness.targets() {
        let result = driver
            .execute_batch(
                &harness.params,
                &queries,
                Some(100),
                1,
                Some("test_schema"),
                None,
            )
            .await
            .unwrap_or_else(|e| panic!("execute_batch failed on {}: {}", target, e));
        let json = serde_json::to_value(&result).expect("serialize batch result");
        normalized_results.push((target.to_string(), normalize_batch_result(&json)));
    }

    for window in normalized_results.windows(2) {
        assert_eq!(
            window[0].1, window[1].1,
            "execute_batch:mixed_statements parity failure between {} and {}",
            window[0].0, window[1].0
        );
    }

    let arr = normalized_results[0].1.as_array().unwrap();
    assert_eq!(arr.len(), 2, "should have results for both statements");
    let first_ok = arr[0].get("error").map(Value::is_null).unwrap_or(false);
    assert!(first_ok, "SELECT should succeed, got: {:?}", arr[0]);
    let second_ok = arr[1].get("error").map(Value::is_null).unwrap_or(false);
    assert!(second_ok, "INSERT should succeed, got: {:?}", arr[1]);
}

#[tokio::test]
#[ignore]
async fn parity_batch_error_handling() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let queries = vec![
        "SELECT 1 AS ok".to_string(),
        "SELECT * FROM test_schema.this_table_does_not_exist".to_string(),
    ];

    let mut normalized_results = Vec::new();
    for (target, driver) in harness.targets() {
        let result = driver
            .execute_batch(
                &harness.params,
                &queries,
                Some(100),
                1,
                Some("test_schema"),
                None,
            )
            .await
            .unwrap_or_else(|e| panic!("execute_batch failed on {}: {}", target, e));
        let json = serde_json::to_value(&result).expect("serialize batch result");
        normalized_results.push((target.to_string(), normalize_batch_result(&json)));
    }

    // Do not assert_eq the error message text across targets — the builtin
    // and plugin surface different underlying driver error strings for the
    // same failure (e.g. differing wording from sqlx vs tokio-postgres).
    // Compare only the success/failure shape.
    for (target, json) in &normalized_results {
        let arr = json.as_array().unwrap();
        assert_eq!(
            arr.len(),
            2,
            "{}: should have results for both statements",
            target
        );
        let first_ok = arr[0].get("error").map(Value::is_null).unwrap_or(false);
        assert!(
            first_ok,
            "{}: valid SELECT should succeed, got: {:?}",
            target, arr[0]
        );
        let second_failed = arr[1].get("error").map(|e| !e.is_null()).unwrap_or(false);
        assert!(
            second_failed,
            "{}: query on non-existent table should fail, got: {:?}",
            target, arr[1]
        );
    }
}
