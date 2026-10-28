//! View management tests.

use crate::helpers::pg_params;
use superdba_lib::drivers::postgres;

#[tokio::test]
#[ignore]
async fn test_get_views() {
    require_pg!();
    let params = pg_params();

    let views = postgres::get_views(&params, "test_schema")
        .await
        .expect("get_views should succeed");

    let view_names: Vec<&str> = views.iter().map(|v| v.name.as_str()).collect();
    assert!(
        view_names.contains(&"active_users"),
        "Expected active_users view, got: {:?}",
        view_names
    );
}

#[tokio::test]
#[ignore]
async fn test_get_view_definition() {
    require_pg!();
    let params = pg_params();

    let def = postgres::get_view_definition(&params, "active_users", "test_schema")
        .await
        .expect("get_view_definition should succeed");

    assert!(
        def.to_lowercase().contains("select"),
        "View definition should contain SELECT, got: {}",
        def
    );
    assert!(
        def.to_lowercase().contains("all_types"),
        "View definition should reference all_types table"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_view_columns() {
    require_pg!();
    let params = pg_params();

    let columns = postgres::get_view_columns(&params, "active_users", "test_schema")
        .await
        .expect("get_view_columns should succeed");

    let col_names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(col_names.contains(&"id"), "Expected id column in view");
    assert!(col_names.contains(&"name"), "Expected name column in view");
    assert!(
        col_names.contains(&"is_active"),
        "Expected is_active column in view"
    );
}

#[tokio::test]
#[ignore]
async fn test_create_and_drop_view() {
    require_pg!();
    let params = pg_params();

    let view_name = "test_temp_view";
    let schema = "test_schema";
    let definition = "SELECT id, col_text FROM test_schema.all_types WHERE id < 10";

    // Create
    postgres::create_view(&params, view_name, definition, schema)
        .await
        .expect("create_view should succeed");

    // Verify exists
    let views = postgres::get_views(&params, schema).await.unwrap();
    assert!(
        views.iter().any(|v| v.name == view_name),
        "Created view should appear in list"
    );

    // Drop
    postgres::drop_view(&params, view_name, schema)
        .await
        .expect("drop_view should succeed");

    // Verify gone
    let views = postgres::get_views(&params, schema).await.unwrap();
    assert!(
        !views.iter().any(|v| v.name == view_name),
        "Dropped view should not appear in list"
    );
}

#[tokio::test]
#[ignore]
async fn test_alter_view() {
    require_pg!();
    let params = pg_params();

    let view_name = "test_alter_view";
    let schema = "test_schema";

    // Cleanup from any prior failed run
    let _ = postgres::drop_view(&params, view_name, schema).await;

    // Create initial view
    let def1 = "SELECT id FROM test_schema.all_types";
    crate::helpers::retry(|| {
        let p = params.clone();
        async move { postgres::create_view(&p, view_name, def1, schema).await }
    })
    .await
    .expect("create_view should succeed");

    // Alter (replace) with new definition
    let def2 = "SELECT id, col_text FROM test_schema.all_types";
    crate::helpers::retry(|| {
        let p = params.clone();
        async move { postgres::alter_view(&p, view_name, def2, schema).await }
    })
    .await
    .expect("alter_view should succeed");

    // Verify new definition has both columns
    let columns = crate::helpers::retry(|| {
        let p = params.clone();
        async move { postgres::get_view_columns(&p, view_name, schema).await }
    })
    .await
    .unwrap();
    assert_eq!(columns.len(), 2, "Altered view should have 2 columns");

    // Cleanup
    postgres::drop_view(&params, view_name, schema)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore]
async fn test_get_views_empty_schema() {
    require_pg!();
    let params = pg_params();

    // other_schema has no views
    let views = postgres::get_views(&params, "other_schema")
        .await
        .expect("get_views should succeed for schema with no views");

    assert!(views.is_empty(), "other_schema should have no views");
}
