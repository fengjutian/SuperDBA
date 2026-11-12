//! Authorization boundary for AI-initiated data access.
//!
//! This module starts with the ChatDBA SQL baseline policy. The public request and decision
//! contract is intentionally independent from transports and database drivers so later
//! policy engines can replace the baseline without changing callers.

pub mod policy;
mod resources;
mod types;

pub use resources::{extract_sql_access, extract_sql_resources, SqlAccess};
pub use types::{
    RiskLevel, SecurityDecision, SecurityOperation, SecurityRequest, SecurityRestrictions,
    SecuritySubject, SecurityVerdict,
};

use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

pub const BASELINE_POLICY_VERSION: &str = "chat-dba-baseline-v1";
pub const MAX_CHAT_ROWS: u32 = 50;
pub const MAX_CHAT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct SecurityEngine;

impl SecurityEngine {
    pub fn authorize_chat_sql(request: &SecurityRequest) -> SecurityDecision {
        if let Some(denial) = validate_request(request) {
            return denial;
        }

        let restrictions = baseline_restrictions(request);

        match request.operation {
            SecurityOperation::SqlRead => decision(
                SecurityVerdict::Allow,
                RiskLevel::L0,
                restrictions,
                "SECURITY_BASELINE_READ_ALLOWED",
            ),
            SecurityOperation::SqlWrite { approved: false } => decision(
                SecurityVerdict::RequireApproval,
                RiskLevel::L2,
                restrictions,
                "SECURITY_WRITE_APPROVAL_REQUIRED",
            ),
            SecurityOperation::SqlWrite { approved: true } => decision(
                SecurityVerdict::Allow,
                RiskLevel::L2,
                restrictions,
                "SECURITY_APPROVED_WRITE_ALLOWED",
            ),
        }
    }

    pub fn authorize_with_policy(
        request: &SecurityRequest,
        policy: &policy::PolicySet,
    ) -> SecurityDecision {
        if let Some(denial) = validate_request(request) {
            return denial;
        }
        if let Err(_error) = policy::validate_policy(policy) {
            return decision_with_version(
                SecurityVerdict::Deny,
                RiskLevel::L3,
                SecurityRestrictions::none(),
                "SECURITY_POLICY_INVALID",
                policy.version.clone(),
            );
        }
        let evaluation = policy::evaluate(policy, request, baseline_restrictions(request));
        decision_with_version(
            evaluation.verdict,
            evaluation.risk,
            evaluation.restrictions,
            evaluation.reason_code,
            policy.version.clone(),
        )
    }

    pub fn authorize_with_local_policy(
        request: &SecurityRequest,
        config_dir: &Path,
    ) -> SecurityDecision {
        match policy::load_local_policy(config_dir) {
            Ok(Some(policy)) => Self::authorize_with_policy(request, &policy),
            Ok(None) => Self::authorize_chat_sql(request),
            Err(_error) => decision(
                SecurityVerdict::Deny,
                RiskLevel::L3,
                SecurityRestrictions::none(),
                "SECURITY_POLICY_LOAD_FAILED",
            ),
        }
    }
}

fn validate_request(request: &SecurityRequest) -> Option<SecurityDecision> {
    if request.request_id.trim().is_empty()
        || request.task_id.trim().is_empty()
        || request.purpose.trim().is_empty()
        || request.connection_id.trim().is_empty()
        || request.subject.user_id.trim().is_empty()
        || request.subject.agent_id.trim().is_empty()
        || request.subject.service_id.trim().is_empty()
    {
        return Some(decision(
            SecurityVerdict::Deny,
            RiskLevel::L3,
            SecurityRestrictions::none(),
            "SECURITY_IDENTITY_INCOMPLETE",
        ));
    }
    if request.requested_max_rows == 0 || request.hard_max_rows == 0 {
        return Some(decision(
            SecurityVerdict::Deny,
            RiskLevel::L3,
            SecurityRestrictions::none(),
            "SECURITY_INVALID_ROW_LIMIT",
        ));
    }
    if request.requested_timeout.is_zero() || request.hard_timeout.is_zero() {
        return Some(decision(
            SecurityVerdict::Deny,
            RiskLevel::L3,
            SecurityRestrictions::none(),
            "SECURITY_INVALID_TIMEOUT",
        ));
    }
    None
}

fn baseline_restrictions(request: &SecurityRequest) -> SecurityRestrictions {
    SecurityRestrictions {
        max_rows: request.requested_max_rows.min(request.hard_max_rows),
        timeout: request.requested_timeout.min(request.hard_timeout),
    }
}

fn decision(
    verdict: SecurityVerdict,
    risk: RiskLevel,
    restrictions: SecurityRestrictions,
    reason_code: &'static str,
) -> SecurityDecision {
    decision_with_version(
        verdict,
        risk,
        restrictions,
        reason_code,
        BASELINE_POLICY_VERSION.to_string(),
    )
}

fn decision_with_version(
    verdict: SecurityVerdict,
    risk: RiskLevel,
    restrictions: SecurityRestrictions,
    reason_code: &'static str,
    policy_version: String,
) -> SecurityDecision {
    SecurityDecision {
        decision_id: Uuid::new_v4(),
        verdict,
        risk,
        policy_version,
        restrictions,
        reason_code,
    }
}

#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod tests;
