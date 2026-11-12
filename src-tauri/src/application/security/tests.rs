use super::*;

fn request(operation: SecurityOperation) -> SecurityRequest {
    SecurityRequest {
        request_id: "request-1".to_string(),
        task_id: "task-1".to_string(),
        subject: SecuritySubject {
            user_id: "local-user".to_string(),
            tenant_id: None,
            agent_id: "chat-dba".to_string(),
            service_id: "superdba".to_string(),
            session_id: None,
        },
        connection_id: "connection-1".to_string(),
        operation,
        requested_max_rows: 500,
        requested_timeout: Duration::from_secs(30),
    }
}

#[test]
fn read_is_allowed_with_hard_limits() {
    let decision = SecurityEngine::authorize_chat_sql(&request(SecurityOperation::SqlRead));

    assert_eq!(decision.verdict, SecurityVerdict::Allow);
    assert_eq!(decision.risk, RiskLevel::L0);
    assert_eq!(decision.restrictions.max_rows, MAX_CHAT_ROWS);
    assert_eq!(decision.restrictions.timeout, MAX_CHAT_TIMEOUT);
}

#[test]
fn unapproved_write_requires_approval() {
    let decision = SecurityEngine::authorize_chat_sql(&request(SecurityOperation::SqlWrite {
        approved: false,
    }));

    assert_eq!(decision.verdict, SecurityVerdict::RequireApproval);
    assert_eq!(decision.risk, RiskLevel::L2);
    assert_eq!(decision.reason_code, "SECURITY_WRITE_APPROVAL_REQUIRED");
}

#[test]
fn approved_write_is_allowed() {
    let decision = SecurityEngine::authorize_chat_sql(&request(SecurityOperation::SqlWrite {
        approved: true,
    }));

    assert_eq!(decision.verdict, SecurityVerdict::Allow);
    assert_eq!(decision.risk, RiskLevel::L2);
}

#[test]
fn incomplete_identity_is_denied() {
    let mut request = request(SecurityOperation::SqlRead);
    request.subject.user_id.clear();

    let decision = SecurityEngine::authorize_chat_sql(&request);

    assert_eq!(decision.verdict, SecurityVerdict::Deny);
    assert_eq!(decision.risk, RiskLevel::L3);
    assert_eq!(decision.reason_code, "SECURITY_IDENTITY_INCOMPLETE");
}

#[test]
fn zero_row_limit_is_denied() {
    let mut request = request(SecurityOperation::SqlRead);
    request.requested_max_rows = 0;

    let decision = SecurityEngine::authorize_chat_sql(&request);

    assert_eq!(decision.verdict, SecurityVerdict::Deny);
    assert_eq!(decision.reason_code, "SECURITY_INVALID_ROW_LIMIT");
}
