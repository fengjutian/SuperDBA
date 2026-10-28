//! Parity integration tests — run driver methods through the harness to prove
//! equivalence across driver implementations.
//!
//! In Phase 0 these serve as a structural validation that the harness works
//! correctly with the built-in driver. In Phase 1 they become the gate: the
//! plugin must produce identical outputs.

use crate::parity::ParityHarness;

#[tokio::test]
#[ignore]
async fn parity_get_schemas() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_schemas", |driver, params| async move {
            driver.get_schemas(&params).await
        })
        .await;

    let schemas: Vec<String> = serde_json::from_value(result).unwrap();
    assert!(schemas.contains(&"test_schema".to_string()));
    assert!(schemas.contains(&"public".to_string()));
}

#[tokio::test]
#[ignore]
async fn parity_get_databases() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_databases", |driver, params| async move {
            driver.get_databases(&params).await
        })
        .await;

    let databases: Vec<String> = serde_json::from_value(result).unwrap();
    assert!(databases.contains(&"testdb".to_string()));
}

#[tokio::test]
#[ignore]
async fn parity_get_tables() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_tables", |driver, params| async move {
            driver.get_tables(&params, Some("test_schema")).await
        })
        .await;

    let arr = result.as_array().expect("tables should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|t| t.get("name")?.as_str()).collect();
    assert!(names.contains(&"all_types"));
    assert!(names.contains(&"orders"));
}

#[tokio::test]
#[ignore]
async fn parity_get_columns() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_columns:all_types", |driver, params| async move {
            driver
                .get_columns(&params, "all_types", Some("test_schema"))
                .await
        })
        .await;

    let arr = result.as_array().expect("columns should be an array");
    assert!(!arr.is_empty());
    let id_col = arr
        .iter()
        .find(|c| c.get("name").and_then(|n| n.as_str()) == Some("id"));
    assert!(id_col.is_some(), "should have an 'id' column");
    assert_eq!(
        id_col.unwrap().get("is_pk").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[tokio::test]
#[ignore]
async fn parity_get_foreign_keys() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_foreign_keys:orders", |driver, params| async move {
            driver
                .get_foreign_keys(&params, "orders", Some("test_schema"))
                .await
        })
        .await;

    let arr = result.as_array().expect("foreign keys should be an array");
    assert!(!arr.is_empty());
    let fk = &arr[0];
    assert_eq!(
        fk.get("column_name").and_then(|v| v.as_str()),
        Some("user_id")
    );
}

#[tokio::test]
#[ignore]
async fn parity_get_indexes() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_indexes:all_types", |driver, params| async move {
            driver
                .get_indexes(&params, "all_types", Some("test_schema"))
                .await
        })
        .await;

    let arr = result.as_array().expect("indexes should be an array");
    assert!(!arr.is_empty());
}

#[tokio::test]
#[ignore]
async fn parity_get_views() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_views", |driver, params| async move {
            driver.get_views(&params, Some("test_schema")).await
        })
        .await;

    let arr = result.as_array().expect("views should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|v| v.get("name")?.as_str()).collect();
    assert!(names.contains(&"active_users"));
}

#[tokio::test]
#[ignore]
async fn parity_get_view_definition() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity(
            "get_view_definition:active_users",
            |driver, params| async move {
                driver
                    .get_view_definition(&params, "active_users", Some("test_schema"))
                    .await
            },
        )
        .await;

    let def = result.as_str().expect("view definition should be a string");
    assert!(def.to_lowercase().contains("select"));
}

#[tokio::test]
#[ignore]
async fn parity_get_materialized_views() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_materialized_views", |driver, params| async move {
            driver
                .get_materialized_views(&params, Some("test_schema"))
                .await
        })
        .await;

    let arr = result
        .as_array()
        .expect("materialized views should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|v| v.get("name")?.as_str()).collect();
    assert!(names.contains(&"user_stats"));
}

#[tokio::test]
#[ignore]
async fn parity_get_routines() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_routines", |driver, params| async move {
            driver.get_routines(&params, Some("test_schema")).await
        })
        .await;

    let arr = result.as_array().expect("routines should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|r| r.get("name")?.as_str()).collect();
    assert!(names.contains(&"add_numbers"));
}

#[tokio::test]
#[ignore]
async fn parity_get_triggers() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity("get_triggers", |driver, params| async move {
            driver.get_triggers(&params, Some("test_schema")).await
        })
        .await;

    let arr = result.as_array().expect("triggers should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|t| t.get("name")?.as_str()).collect();
    assert!(names.contains(&"trg_audit"));
}

#[tokio::test]
#[ignore]
async fn parity_get_tables_secondary() {
    require_pg!();
    let harness = ParityHarness::new().await;

    let result = harness
        .assert_parity_secondary("get_tables:secondary", |driver, params| async move {
            driver.get_tables(&params, Some("secondary_schema")).await
        })
        .await;

    let arr = result.as_array().expect("tables should be an array");
    let names: Vec<&str> = arr.iter().filter_map(|t| t.get("name")?.as_str()).collect();
    assert!(names.contains(&"remote_data"));
}

#[tokio::test]
#[ignore]
async fn parity_map_inferred_type() {
    require_pg!();
    let harness = ParityHarness::new().await;

    // map_inferred_type is synchronous — test it directly on each target
    for (target, driver) in harness.targets() {
        assert_eq!(
            driver.map_inferred_type("DATETIME"),
            "TIMESTAMP",
            "map_inferred_type(DATETIME) failed on {}",
            target
        );
        assert_eq!(
            driver.map_inferred_type("JSON"),
            "JSONB",
            "map_inferred_type(JSON) failed on {}",
            target
        );
        assert_eq!(
            driver.map_inferred_type("TEXT"),
            "TEXT",
            "map_inferred_type(TEXT) passthrough failed on {}",
            target
        );
    }
}
