use super::policy::*;
use super::*;
use std::fs;
use tempfile::tempdir;

fn request(operation: SecurityOperation) -> SecurityRequest {
    SecurityRequest {
        request_id: "request-1".to_string(),
        task_id: "task-1".to_string(),
        purpose: "sales-analysis".to_string(),
        subject: SecuritySubject {
            user_id: "user-1".to_string(),
            roles: vec!["sales".to_string()],
            tenant_id: Some("tenant-1".to_string()),
            agent_id: "chat-dba".to_string(),
            service_id: "superdba".to_string(),
            session_id: None,
        },
        connection_id: "crm".to_string(),
        operation,
        requested_max_rows: 500,
        requested_timeout: Duration::from_secs(30),
        hard_max_rows: 1_000,
        hard_timeout: Duration::from_secs(30),
    }
}

fn rule(id: &str, effect: PolicyEffect) -> PolicyRule {
    PolicyRule {
        id: id.to_string(),
        effect,
        users: Vec::new(),
        roles: vec!["sales".to_string()],
        tenants: vec!["tenant-1".to_string()],
        agents: vec!["chat-dba".to_string()],
        services: vec!["superdba".to_string()],
        connections: vec!["crm".to_string()],
        purposes: vec!["sales-analysis".to_string()],
        actions: vec![PolicyAction::SqlRead],
        limits: None,
    }
}

fn policy(rules: Vec<PolicyRule>) -> PolicySet {
    PolicySet {
        version: "v1".to_string(),
        default: PolicyEffect::Deny,
        rules,
    }
}

#[test]
fn matching_allow_rule_grants_read_and_tightens_limits() {
    let mut allow = rule("allow-sales", PolicyEffect::Allow);
    allow.limits = Some(PolicyLimits {
        max_rows: Some(25),
        timeout_ms: Some(2_000),
    });

    let decision = SecurityEngine::authorize_with_policy(
        &request(SecurityOperation::SqlRead),
        &policy(vec![allow]),
    );

    assert_eq!(decision.verdict, SecurityVerdict::Allow);
    assert_eq!(decision.restrictions.max_rows, 25);
    assert_eq!(decision.restrictions.timeout, Duration::from_secs(2));
    assert_eq!(decision.policy_version, "v1");
}

#[test]
fn unmatched_request_is_denied_by_default() {
    let mut request = request(SecurityOperation::SqlRead);
    request.purpose = "customer-profiling".to_string();

    let decision = SecurityEngine::authorize_with_policy(
        &request,
        &policy(vec![rule("allow-sales", PolicyEffect::Allow)]),
    );

    assert_eq!(decision.verdict, SecurityVerdict::Deny);
    assert_eq!(decision.reason_code, "SECURITY_POLICY_DEFAULT_DENY");
}

#[test]
fn explicit_deny_overrides_allow() {
    let mut deny = rule("deny-user", PolicyEffect::Deny);
    deny.users = vec!["user-1".to_string()];

    let decision = SecurityEngine::authorize_with_policy(
        &request(SecurityOperation::SqlRead),
        &policy(vec![rule("allow-sales", PolicyEffect::Allow), deny]),
    );

    assert_eq!(decision.verdict, SecurityVerdict::Deny);
    assert_eq!(decision.reason_code, "SECURITY_POLICY_EXPLICIT_DENY");
}

#[test]
fn write_policy_still_requires_approval() {
    let mut allow = rule("allow-write", PolicyEffect::Allow);
    allow.actions = vec![PolicyAction::SqlWrite];

    let decision = SecurityEngine::authorize_with_policy(
        &request(SecurityOperation::SqlWrite { approved: false }),
        &policy(vec![allow]),
    );

    assert_eq!(decision.verdict, SecurityVerdict::RequireApproval);
}

#[test]
fn validation_rejects_duplicate_rules_and_allow_by_default() {
    let duplicate = rule("same", PolicyEffect::Allow);
    assert!(validate_policy(&policy(vec![duplicate.clone(), duplicate])).is_err());

    let mut unsafe_default = policy(Vec::new());
    unsafe_default.default = PolicyEffect::Allow;
    assert!(validate_policy(&unsafe_default).is_err());
}

#[test]
fn loading_policy_adds_content_digest_to_version() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("security-policy.yaml");
    fs::write(
        &path,
        r#"version: v1
default: deny
rules:
  - id: allow-read
    effect: allow
    roles: [sales]
    actions: [sql_read]
"#,
    )
    .unwrap();

    let loaded = load_policy(&path).unwrap();

    assert!(loaded.version.starts_with("v1:"));
    assert_eq!(loaded.version.len(), 19);
}

#[test]
fn local_policy_uses_last_valid_version_after_invalid_replacement() {
    let directory = tempdir().unwrap();
    let path = directory.path().join(LOCAL_POLICY_FILE);
    fs::write(
        &path,
        r#"version: v1
default: deny
rules:
  - id: allow-read
    effect: allow
    roles: [sales]
    actions: [sql_read]
"#,
    )
    .unwrap();
    let first = load_local_policy(directory.path())
        .unwrap()
        .expect("policy should load");

    fs::write(&path, "not: [valid").unwrap();
    let cached = load_local_policy(directory.path())
        .unwrap()
        .expect("last valid policy should remain available");

    assert_eq!(cached.version, first.version);
}

#[test]
fn invalid_first_local_policy_fails_closed() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join(LOCAL_POLICY_FILE), "not: [valid").unwrap();

    let error = load_local_policy(directory.path()).expect_err("invalid policy must fail");

    assert!(error.contains("Invalid security policy"));
}

#[test]
fn deleting_local_policy_returns_to_baseline() {
    let directory = tempdir().unwrap();
    let path = directory.path().join(LOCAL_POLICY_FILE);
    fs::write(&path, "version: v1\ndefault: deny\nrules: []\n").unwrap();
    assert!(load_local_policy(directory.path()).unwrap().is_some());

    fs::remove_file(path).unwrap();

    assert!(load_local_policy(directory.path()).unwrap().is_none());
}

#[test]
fn runtime_authorization_applies_local_policy() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join(LOCAL_POLICY_FILE),
        r#"version: runtime-v1
default: deny
rules:
  - id: allow-sales-read
    effect: allow
    roles: [sales]
    agents: [chat-dba]
    connections: [crm]
    purposes: [sales-analysis]
    actions: [sql_read]
    limits:
      max_rows: 7
"#,
    )
    .unwrap();

    let decision = SecurityEngine::authorize_with_local_policy(
        &request(SecurityOperation::SqlRead),
        directory.path(),
    );

    assert_eq!(decision.verdict, SecurityVerdict::Allow);
    assert_eq!(decision.restrictions.max_rows, 7);
    assert!(decision.policy_version.starts_with("runtime-v1:"));
}
