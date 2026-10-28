//! Parity tests for `explain_query` — verifies both drivers succeed on EXPLAIN.
//!
//! Note: ExplainQueryOutput differs structurally between built-in (Raw variant)
//! and plugin (Plan variant). These tests verify that both drivers return Ok
//! (no error) rather than comparing exact output, since EXPLAIN output contains
//! volatile runtime values (cost estimates, actual times, buffers).

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_explain_simple() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // We cannot use assert_parity here because the output format differs.
    // Instead, verify that each target returns Ok (non-error) for EXPLAIN.
    for (target, driver) in harness.targets() {
        let result = driver
            .explain_query(
                &harness.params,
                "SELECT id, col_text FROM test_schema.all_types WHERE id < 5",
                false,
                Some("test_schema"),
            )
            .await;

        assert!(
            result.is_ok(),
            "EXPLAIN (no analyze) failed on target {}: {:?}",
            target,
            result.err()
        );
    }
}

#[tokio::test]
#[ignore]
async fn parity_explain_analyze() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // EXPLAIN ANALYZE actually executes the query and reports timing.
    for (target, driver) in harness.targets() {
        let result = driver
            .explain_query(
                &harness.params,
                "SELECT id FROM test_schema.all_types ORDER BY id LIMIT 3",
                true,
                Some("test_schema"),
            )
            .await;

        assert!(
            result.is_ok(),
            "EXPLAIN ANALYZE failed on target {}: {:?}",
            target,
            result.err()
        );
    }
}
