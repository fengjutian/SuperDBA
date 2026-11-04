use super::*;

#[tokio::test]
async fn serializes_ai_work_per_connection() {
    let connection_id = format!("ai-guard-{}", uuid::Uuid::new_v4());
    let first = acquire(&connection_id).await.expect("first permit");
    let second = semaphore(&connection_id);

    assert!(second.clone().try_acquire_owned().is_err());
    drop(first);
    assert!(second.try_acquire_owned().is_ok());
}

#[tokio::test]
async fn bounds_ai_queries_across_connections() {
    let global = Arc::new(Semaphore::new(1));
    let first = acquire_ai_permits(
        global.clone(),
        Arc::new(Semaphore::new(1)),
        Duration::from_millis(10),
    )
    .await
    .expect("first global permit");

    let error = acquire_ai_permits(
        global,
        Arc::new(Semaphore::new(1)),
        Duration::from_millis(1),
    )
    .await
    .expect_err("a second connection must respect the global limit");
    assert_eq!(error, "Timed out waiting for AI query capacity.");

    drop(first);
}

#[test]
fn mcp_row_limits_are_bounded() {
    assert!(MCP_DEFAULT_ROWS > 0);
    assert!(MCP_DEFAULT_ROWS <= MCP_MAX_ROWS);
}

#[tokio::test]
async fn bounds_total_database_workloads_without_fully_serializing_the_editor() {
    let connection_id = format!("workload-guard-{}", uuid::Uuid::new_v4());
    let first = acquire_workload(&connection_id)
        .await
        .expect("first permit");
    let second = acquire_workload(&connection_id)
        .await
        .expect("second permit");
    let semaphore = workload_semaphore(&connection_id);

    assert!(semaphore.clone().try_acquire_owned().is_err());
    drop(first);
    assert!(semaphore.try_acquire_owned().is_ok());
    drop(second);
}

#[tokio::test]
async fn scheduler_wait_is_bounded() {
    let semaphore = Arc::new(Semaphore::new(0));
    let error = acquire_with_timeout(semaphore, Duration::from_millis(1), "queue timeout")
        .await
        .expect_err("a saturated scheduler must time out");
    assert_eq!(error, "queue timeout");
}

#[test]
fn repeated_failures_open_and_success_resets_the_circuit() {
    let connection_id = format!("circuit-guard-{}", uuid::Uuid::new_v4());

    for _ in 0..CIRCUIT_FAILURE_THRESHOLD - 1 {
        record_query_failure(&connection_id);
        assert!(ensure_circuit_closed(&connection_id).is_ok());
    }
    record_query_failure(&connection_id);
    assert!(ensure_circuit_closed(&connection_id).is_err());

    record_query_success(&connection_id);
    assert!(ensure_circuit_closed(&connection_id).is_ok());
}

#[test]
fn expired_circuit_allows_queries_again() {
    let connection_id = format!("expired-circuit-{}", uuid::Uuid::new_v4());
    QUERY_CIRCUITS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(
            connection_id.clone(),
            CircuitState {
                consecutive_failures: CIRCUIT_FAILURE_THRESHOLD,
                last_failure: Instant::now(),
                open_until: Some(Instant::now() - Duration::from_millis(1)),
            },
        );

    assert!(ensure_circuit_closed(&connection_id).is_ok());
}
