use super::{RiskLevel, SecurityOperation, SecurityRequest, SecurityRestrictions, SecurityVerdict};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

const MAX_POLICY_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    SqlRead,
    SqlWrite,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLimits {
    pub max_rows: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub id: String,
    pub effect: PolicyEffect,
    #[serde(default)]
    pub users: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub tenants: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub connections: Vec<String>,
    #[serde(default)]
    pub purposes: Vec<String>,
    #[serde(default)]
    pub actions: Vec<PolicyAction>,
    pub limits: Option<PolicyLimits>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySet {
    pub version: String,
    pub default: PolicyEffect,
    pub rules: Vec<PolicyRule>,
}

pub(crate) struct PolicyEvaluation {
    pub verdict: SecurityVerdict,
    pub risk: RiskLevel,
    pub restrictions: SecurityRestrictions,
    pub reason_code: &'static str,
}

pub fn load_policy(path: &Path) -> Result<PolicySet, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Failed to read security policy metadata: {error}"))?;
    if metadata.len() > MAX_POLICY_BYTES {
        return Err(format!(
            "Security policy exceeds the {MAX_POLICY_BYTES} byte limit"
        ));
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read security policy: {error}"))?;
    let mut policy: PolicySet = serde_yaml::from_str(&content)
        .map_err(|error| format!("Invalid security policy: {error}"))?;
    validate_policy(&policy)?;
    let digest = Sha256::digest(content.as_bytes());
    let digest = format!("{digest:x}");
    policy.version = format!("{}:{}", policy.version, &digest[..16]);
    Ok(policy)
}

pub fn validate_policy(policy: &PolicySet) -> Result<(), String> {
    if policy.version.trim().is_empty() {
        return Err("Security policy version must not be empty".to_string());
    }
    if policy.default != PolicyEffect::Deny {
        return Err("Security policy default must be deny".to_string());
    }
    let mut ids = HashSet::new();
    for rule in &policy.rules {
        if rule.id.trim().is_empty() {
            return Err("Security policy rule id must not be empty".to_string());
        }
        if !ids.insert(rule.id.as_str()) {
            return Err(format!("Duplicate security policy rule id: {}", rule.id));
        }
        if rule.actions.is_empty() {
            return Err(format!(
                "Security policy rule {} must declare at least one action",
                rule.id
            ));
        }
        if let Some(limits) = &rule.limits {
            if limits.max_rows == Some(0) || limits.timeout_ms == Some(0) {
                return Err(format!(
                    "Security policy rule {} contains a zero limit",
                    rule.id
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn evaluate(
    policy: &PolicySet,
    request: &SecurityRequest,
    baseline: SecurityRestrictions,
) -> PolicyEvaluation {
    let matching = policy
        .rules
        .iter()
        .filter(|rule| matches_rule(rule, request))
        .collect::<Vec<_>>();

    if matching
        .iter()
        .any(|rule| rule.effect == PolicyEffect::Deny)
    {
        return denied("SECURITY_POLICY_EXPLICIT_DENY");
    }

    let allows = matching
        .into_iter()
        .filter(|rule| rule.effect == PolicyEffect::Allow)
        .collect::<Vec<_>>();
    if allows.is_empty() {
        return denied("SECURITY_POLICY_DEFAULT_DENY");
    }

    let restrictions = allows.iter().fold(baseline, |current, rule| {
        let Some(limits) = &rule.limits else {
            return current;
        };
        SecurityRestrictions {
            max_rows: limits
                .max_rows
                .map_or(current.max_rows, |limit| current.max_rows.min(limit)),
            timeout: limits.timeout_ms.map_or(current.timeout, |timeout_ms| {
                current.timeout.min(Duration::from_millis(timeout_ms))
            }),
        }
    });

    match request.operation {
        SecurityOperation::SqlRead => PolicyEvaluation {
            verdict: SecurityVerdict::Allow,
            risk: RiskLevel::L0,
            restrictions,
            reason_code: "SECURITY_POLICY_ALLOWED",
        },
        SecurityOperation::SqlWrite { approved: false } => PolicyEvaluation {
            verdict: SecurityVerdict::RequireApproval,
            risk: RiskLevel::L2,
            restrictions,
            reason_code: "SECURITY_WRITE_APPROVAL_REQUIRED",
        },
        SecurityOperation::SqlWrite { approved: true } => PolicyEvaluation {
            verdict: SecurityVerdict::Allow,
            risk: RiskLevel::L2,
            restrictions,
            reason_code: "SECURITY_POLICY_ALLOWED",
        },
    }
}

fn matches_rule(rule: &PolicyRule, request: &SecurityRequest) -> bool {
    matches_selector(&rule.users, &request.subject.user_id)
        && matches_any_selector(&rule.roles, &request.subject.roles)
        && matches_optional_selector(&rule.tenants, request.subject.tenant_id.as_deref())
        && matches_selector(&rule.agents, &request.subject.agent_id)
        && matches_selector(&rule.services, &request.subject.service_id)
        && matches_selector(&rule.connections, &request.connection_id)
        && matches_selector(&rule.purposes, &request.purpose)
        && rule.actions.contains(&operation_action(request.operation))
}

fn matches_selector(selector: &[String], value: &str) -> bool {
    selector.is_empty() || selector.iter().any(|candidate| candidate == value)
}

fn matches_optional_selector(selector: &[String], value: Option<&str>) -> bool {
    selector.is_empty() || value.is_some_and(|value| matches_selector(selector, value))
}

fn matches_any_selector(selector: &[String], values: &[String]) -> bool {
    selector.is_empty()
        || values
            .iter()
            .any(|value| selector.iter().any(|candidate| candidate == value))
}

fn operation_action(operation: SecurityOperation) -> PolicyAction {
    match operation {
        SecurityOperation::SqlRead => PolicyAction::SqlRead,
        SecurityOperation::SqlWrite { .. } => PolicyAction::SqlWrite,
    }
}

fn denied(reason_code: &'static str) -> PolicyEvaluation {
    PolicyEvaluation {
        verdict: SecurityVerdict::Deny,
        risk: RiskLevel::L3,
        restrictions: SecurityRestrictions::none(),
        reason_code,
    }
}
