//! EXPLAIN query plan tests.

use crate::helpers::pg_params;
use superdba_lib::drivers::driver_trait::DatabaseDriver;
use superdba_lib::drivers::postgres::PostgresDriver;
use superdba_lib::models::ExplainQueryOutput;

#[tokio::test]
#[ignore]
async fn test_explain_simple_select() {
    require_pg!();
    let params = pg_params();

    let output = PostgresDriver::new()
        .explain_query(
            &params,
            "SELECT * FROM test_schema.all_types WHERE id = 1",
            false,
            Some("test_schema"),
        )
        .await
        .expect("explain_query should succeed");

    match &output {
        ExplainQueryOutput::Plan { plan } => {
            assert!(!plan.is_null(), "Plan should not be null");
        }
        ExplainQueryOutput::Raw { raw } => {
            assert!(!raw.payload.is_empty(), "Raw output should have lines");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_explain_analyze() {
    require_pg!();
    let params = pg_params();

    let output = PostgresDriver::new()
        .explain_query(
            &params,
            "SELECT * FROM test_schema.all_types WHERE id = 1",
            true,
            Some("test_schema"),
        )
        .await
        .expect("explain_query with analyze should succeed");

    match &output {
        ExplainQueryOutput::Plan { plan } => {
            assert!(!plan.is_null(), "ANALYZE plan should not be null");
        }
        ExplainQueryOutput::Raw { raw } => {
            assert!(
                !raw.payload.is_empty(),
                "ANALYZE raw output should have lines"
            );
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_explain_join_query() {
    require_pg!();
    let params = pg_params();

    let output = PostgresDriver::new()
        .explain_query(
            &params,
            "SELECT o.id, oi.product FROM test_schema.orders o \
             JOIN test_schema.order_items oi ON o.id = oi.order_id",
            false,
            Some("test_schema"),
        )
        .await
        .expect("explain JOIN should succeed");

    match &output {
        ExplainQueryOutput::Plan { plan } => {
            assert!(!plan.is_null(), "JOIN plan should not be null");
        }
        ExplainQueryOutput::Raw { raw } => {
            assert!(!raw.payload.is_empty(), "JOIN raw output should have lines");
        }
    }
}
