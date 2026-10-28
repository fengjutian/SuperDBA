//! Extra parity tests for views — alter_view and empty schema scenarios.

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_alter_view() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let view_name = "parity_alter_view";
    let schema = Some("test_schema");
    let def1 = "SELECT id FROM test_schema.all_types";
    let def2 = "SELECT id, col_text FROM test_schema.all_types";

    // create_view/drop_view are destructive against the one shared physical
    // database both targets point at — assert_parity calls each target in
    // sequence, so the second target's plain CREATE VIEW would legitimately
    // fail with "view already exists" and its DROP would legitimately fail
    // with "view does not exist" once the first target already did so.
    // alter_view itself uses CREATE OR REPLACE (idempotent), so it's safe
    // under assert_parity — but the surrounding create/drop are not. Run the
    // whole create->alter->verify->drop sequence directly per target.
    for (target, driver) in harness.targets() {
        // Cleanup from any prior failed run.
        let _ = driver.drop_view(&harness.params, view_name, schema).await;

        driver
            .create_view(&harness.params, view_name, def1, schema)
            .await
            .unwrap_or_else(|e| panic!("create_view failed on {}: {}", target, e));

        driver
            .alter_view(&harness.params, view_name, def2, schema)
            .await
            .unwrap_or_else(|e| panic!("alter_view failed on {}: {}", target, e));

        let columns = driver
            .get_view_columns(&harness.params, view_name, schema)
            .await
            .unwrap_or_else(|e| panic!("get_view_columns failed on {}: {}", target, e));
        assert_eq!(
            columns.len(),
            2,
            "{}: altered view should have 2 columns, got: {}",
            target,
            columns.len()
        );

        driver
            .drop_view(&harness.params, view_name, schema)
            .await
            .unwrap_or_else(|e| panic!("drop_view (cleanup) failed on {}: {}", target, e));
    }
}

#[tokio::test]
#[ignore]
async fn parity_get_views_empty_schema() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // other_schema has no views — both drivers should return an empty list
    let result = harness
        .assert_parity("get_views:empty_schema", |driver, params| async move {
            driver.get_views(&params, Some("other_schema")).await
        })
        .await;

    let views = result.as_array().expect("views should be an array");
    assert!(
        views.is_empty(),
        "other_schema should have no views, got: {:?}",
        views
    );
}
