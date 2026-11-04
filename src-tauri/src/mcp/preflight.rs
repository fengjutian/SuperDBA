//! Pre-flight EXPLAIN dispatch for the MCP approval gate.
//!
//! Wraps the existing per-driver `explain_query` functions and returns the
//! plan as a `serde_json::Value` so it can be embedded in a `PendingApproval`
//! file and rendered by the frontend's Visual Explain component.
//!
//! The wrapper is intentionally non-blocking: if EXPLAIN fails (DDL, missing
//! permission, syntax error) we return the error string and let the caller
//! decide whether to still surface the approval modal.

use serde_json::Value;

use crate::application::ai_query_guard;
use crate::drivers::registry as driver_registry;
use crate::models::ConnectionParams;

/// Result of a pre-flight EXPLAIN attempt.
pub struct PreflightOutcome {
    pub plan: Option<Value>,
    pub error: Option<String>,
}

/// Run EXPLAIN against the given driver and return the plan as JSON.
pub async fn preflight_explain(
    driver: &str,
    params: &ConnectionParams,
    query: &str,
    schema: Option<&str>,
) -> PreflightOutcome {
    let registered = match driver_registry::get_driver(driver).await {
        Some(d) => d,
        None => {
            return PreflightOutcome {
                plan: None,
                error: Some(format!("EXPLAIN not supported for driver: {}", driver)),
            };
        }
    };
    let plan_result = match tokio::time::timeout(
        ai_query_guard::PREFLIGHT_EXPLAIN_TIMEOUT,
        registered.explain_query(params, query, false, schema),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            return PreflightOutcome {
                plan: None,
                error: Some(format!(
                    "Pre-flight EXPLAIN timed out after {} seconds",
                    ai_query_guard::PREFLIGHT_EXPLAIN_TIMEOUT.as_secs()
                )),
            }
        }
    };

    match plan_result {
        Ok(plan) => match serde_json::to_value(&plan) {
            Ok(v) => PreflightOutcome {
                plan: Some(v),
                error: None,
            },
            Err(e) => PreflightOutcome {
                plan: None,
                error: Some(e.to_string()),
            },
        },
        Err(e) => PreflightOutcome {
            plan: None,
            error: Some(e),
        },
    }
}
