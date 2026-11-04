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

#[test]
fn mcp_row_limits_are_bounded() {
    assert!(MCP_DEFAULT_ROWS > 0);
    assert!(MCP_DEFAULT_ROWS <= MCP_MAX_ROWS);
}
