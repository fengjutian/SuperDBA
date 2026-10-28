//! Trigger management tests.

use crate::helpers::pg_params;
use superdba_lib::drivers::postgres;

#[tokio::test]
#[ignore]
async fn test_get_triggers() {
    require_pg!();
    let params = pg_params();

    let triggers = postgres::get_triggers(&params, "test_schema")
        .await
        .expect("get_triggers should succeed");

    let trigger_names: Vec<&str> = triggers.iter().map(|t| t.name.as_str()).collect();
    assert!(
        trigger_names.contains(&"trg_audit"),
        "Expected trg_audit trigger, got: {:?}",
        trigger_names
    );
}

#[tokio::test]
#[ignore]
async fn test_get_trigger_definition() {
    require_pg!();
    let params = pg_params();

    let def = postgres::get_trigger_definition(&params, "trg_audit", "all_types", "test_schema")
        .await
        .expect("get_trigger_definition should succeed");

    assert!(
        def.to_lowercase().contains("after update"),
        "Trigger definition should indicate AFTER UPDATE, got: {}",
        def
    );
    assert!(
        def.to_lowercase().contains("all_types"),
        "Trigger definition should reference all_types table"
    );
}

#[tokio::test]
#[ignore]
async fn test_create_and_drop_trigger() {
    require_pg!();
    let params = pg_params();

    let trigger_name = "trg_test_temp";
    let schema = "test_schema";

    // Cleanup from any prior failed run
    let _ = postgres::drop_trigger(&params, trigger_name, "crud_scratch", schema).await;

    // Create trigger (reuse existing trigger function)
    let create_sql = format!(
        "CREATE TRIGGER {} BEFORE INSERT ON {}.crud_scratch \
         FOR EACH ROW EXECUTE FUNCTION {}.audit_trigger_fn()",
        trigger_name, schema, schema
    );
    postgres::create_trigger(&params, &create_sql, schema)
        .await
        .expect("create_trigger should succeed");

    // Verify exists
    let triggers = postgres::get_triggers(&params, schema).await.unwrap();
    assert!(
        triggers.iter().any(|t| t.name == trigger_name),
        "Created trigger should appear in list"
    );

    // Drop
    postgres::drop_trigger(&params, trigger_name, "crud_scratch", schema)
        .await
        .expect("drop_trigger should succeed");

    // Verify gone
    let triggers = postgres::get_triggers(&params, schema).await.unwrap();
    assert!(
        !triggers.iter().any(|t| t.name == trigger_name),
        "Dropped trigger should not appear in list"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_triggers_empty_schema() {
    require_pg!();
    let params = pg_params();

    let triggers = postgres::get_triggers(&params, "other_schema")
        .await
        .expect("get_triggers should succeed for schema with no triggers");

    assert!(triggers.is_empty(), "other_schema should have no triggers");
}
