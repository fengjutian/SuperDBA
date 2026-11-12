//! Authorization boundary for AI-initiated data access.
//!
//! This module starts with the ChatDBA SQL baseline policy. The public request and decision
//! contract is intentionally independent from transports and database drivers so later
//! policy engines can replace the baseline without changing callers.

mod types;

pub use types::{
    RiskLevel, SecurityDecision, SecurityOperation, SecurityRequest, SecurityRestrictions,
    SecuritySubject, SecurityVerdict,
};

use std::time::Duration;
use uuid::Uuid;

pub const BASELINE_POLICY_VERSION: &str = "chat-dba-baseline-v1";
pub const MAX_CHAT_ROWS: u32 = 50;
pub const MAX_CHAT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct SecurityEngine;

impl SecurityEngine {
    pub fn authorize_chat_sql(request: &SecurityRequest) -> SecurityDecision {
        if request.request_id.trim().is_empty()
            || request.task_id.trim().is_empty()
            || request.connection_id.trim().is_empty()
            || request.subject.user_id.trim().is_empty()
            || request.subject.agent_id.trim().is_empty()
            || request.subject.service_id.trim().is_empty()
        {
            return decision(
                SecurityVerdict::Deny,
                RiskLevel::L3,
                SecurityRestrictions::none(),
                "SECURITY_IDENTITY_INCOMPLETE",
            );
        }

        if request.requested_max_rows == 0 || request.hard_max_rows == 0 {
            return decision(
                SecurityVerdict::Deny,
                RiskLevel::L3,
                SecurityRestrictions::none(),
                "SECURITY_INVALID_ROW_LIMIT",
            );
        }
        if request.requested_timeout.is_zero() || request.hard_timeout.is_zero() {
            return decision(
                SecurityVerdict::Deny,
                RiskLevel::L3,
                SecurityRestrictions::none(),
                "SECURITY_INVALID_TIMEOUT",
            );
        }

        let restrictions = SecurityRestrictions {
            max_rows: request.requested_max_rows.min(request.hard_max_rows),
            timeout: request.requested_timeout.min(request.hard_timeout),
        };

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
}

fn decision(
    verdict: SecurityVerdict,
    risk: RiskLevel,
    restrictions: SecurityRestrictions,
    reason_code: &'static str,
) -> SecurityDecision {
    SecurityDecision {
        decision_id: Uuid::new_v4(),
        verdict,
        risk,
        policy_version: BASELINE_POLICY_VERSION,
        restrictions,
        reason_code,
    }
}

#[cfg(test)]
mod tests;
