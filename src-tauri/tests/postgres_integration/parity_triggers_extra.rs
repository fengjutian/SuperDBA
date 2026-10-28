//! Extra parity tests for triggers — create/drop trigger and empty schema.

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_create_and_drop_trigger() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let trigger_name = "trg_parity_temp";
    let table_name = "crud_scratch";
    let schema = Some("test_schema");

    // Cleanup from any prior failed run
    for (_target, driver) in harness.targets() {
        let _ = driver
            .drop_trigger(&harness.params, trigger_name, table_name, schema)
            .await;
    }

    // create_trigger/drop_trigger are destructive against the one shared
    // physical database both targets point at — assert_parity calls each
    // target in sequence, so the second target's CREATE would legitimately
    // fail with "trigger already exists" (created by the first target) and
    // its DROP would legitimately fail with "trigger does not exist" (already
    // dropped by the first target). Run create+drop directly per target
    // instead, so each target creates its own copy and drops its own copy.
    let create_sql = format!(
        "CREATE TRIGGER {} BEFORE INSERT ON test_schema.{} \
         FOR EACH ROW EXECUTE FUNCTION test_schema.audit_trigger_fn()",
        trigger_name, table_name
    );

    for (target, driver) in harness.targets() {
        driver
            .create_trigger(&harness.params, &create_sql, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("create_trigger failed on {}: {}", target, e));

        // Verify the trigger exists by listing triggers (read-only, safe to
        // check per-target since both point at the same live state).
        let triggers = driver
            .get_triggers(&harness.params, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("get_triggers failed on {}: {}", target, e));
        let found = triggers.iter().any(|t| t.name == trigger_name);
        assert!(
            found,
            "{}: created trigger {} should appear in list",
            target, trigger_name
        );

        driver
            .drop_trigger(
                &harness.params,
                trigger_name,
                table_name,
                Some("test_schema"),
            )
            .await
            .unwrap_or_else(|e| panic!("drop_trigger failed on {}: {}", target, e));

        let triggers = driver
            .get_triggers(&harness.params, Some("test_schema"))
            .await
            .unwrap_or_else(|e| panic!("get_triggers (after drop) failed on {}: {}", target, e));
        let still_found = triggers.iter().any(|t| t.name == trigger_name);
        assert!(
            !still_found,
            "{}: dropped trigger {} should not appear in list",
            target, trigger_name
        );
    }
}

#[tokio::test]
#[ignore]
async fn parity_get_triggers_empty_schema() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // other_schema has no triggers — both drivers should return an empty list
    let result = harness
        .assert_parity("get_triggers:empty_schema", |driver, params| async move {
            driver.get_triggers(&params, Some("other_schema")).await
        })
        .await;

    let triggers = result.as_array().expect("triggers should be an array");
    assert!(
        triggers.is_empty(),
        "other_schema should have no triggers, got: {:?}",
        triggers
    );
}
