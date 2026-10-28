//! Parity tests for view and materialized view lifecycle operations.

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_get_view_columns() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_view_columns:active_users",
            |driver, params| async move {
                driver
                    .get_view_columns(&params, "active_users", Some("test_schema"))
                    .await
            },
        )
        .await;

    let columns = result.as_array().expect("view columns should be an array");
    assert!(!columns.is_empty(), "active_users view should have columns");
}

#[tokio::test]
#[ignore]
async fn parity_create_drop_view() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let view_name = "parity_temp_view";
    let definition = "SELECT id, col_text FROM test_schema.all_types WHERE id < 10";

    // create_view/drop_view are destructive against the one shared physical
    // database both targets point at — assert_parity calls each target in
    // sequence, so the second target's plain CREATE VIEW would legitimately
    // fail with "view already exists" (created by the first target) and its
    // DROP would legitimately fail with "view does not exist" (already
    // dropped by the first target). Run create+verify+drop directly per
    // target instead, so each target creates and drops its own view.
    for (target, driver) in harness.targets() {
        // Cleanup from any prior failed run.
        let _ = driver
            .drop_view(&harness.params, view_name, Some("test_schema"))
            .await;

        driver
            .create_view(&harness.params, view_name, definition, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("create_view failed on {}: {}", target, e));

        let columns = driver
            .get_view_columns(&harness.params, view_name, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("get_view_columns failed on {}: {}", target, e));
        assert!(
            !columns.is_empty(),
            "{}: temp view should have columns",
            target
        );

        driver
            .drop_view(&harness.params, view_name, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("drop_view failed on {}: {}", target, e));

        // get_view_columns queries information_schema.columns filtered by
        // table name, so a dropped view returns Ok(empty), not Err.
        let columns_after_drop = driver
            .get_view_columns(&harness.params, view_name, Some("test_schema"))
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "get_view_columns on dropped view should return Ok(empty), not Err, on {}: {}",
                    target, e
                )
            });
        assert!(
            columns_after_drop.is_empty(),
            "{}: view should have no columns after drop, got: {:?}",
            target,
            columns_after_drop
        );
    }
}

#[tokio::test]
#[ignore]
async fn parity_get_materialized_view_columns() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_materialized_view_columns:user_stats",
            |driver, params| async move {
                driver
                    .get_materialized_view_columns(&params, "user_stats", Some("test_schema"))
                    .await
            },
        )
        .await;

    let columns = result
        .as_array()
        .expect("materialized view columns should be an array");
    assert!(
        !columns.is_empty(),
        "user_stats materialized view should have columns"
    );
}

#[tokio::test]
#[ignore]
async fn parity_refresh_materialized_view() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // refresh_materialized_view returns () on success — verify no error
    harness
        .assert_parity(
            "refresh_materialized_view:user_stats",
            |driver, params| async move {
                driver
                    .refresh_materialized_view(&params, "user_stats", Some("test_schema"))
                    .await
            },
        )
        .await;
}
