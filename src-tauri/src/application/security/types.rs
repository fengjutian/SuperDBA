use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecuritySubject {
    pub user_id: String,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
    pub agent_id: String,
    pub service_id: String,
    pub session_id: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityOperation {
    SqlRead,
    SqlWrite { approved: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityRequest {
    pub request_id: String,
    pub task_id: String,
    pub purpose: String,
    pub subject: SecuritySubject,
    pub connection_id: String,
    pub operation: SecurityOperation,
    pub requested_max_rows: u32,
    pub requested_timeout: Duration,
    pub hard_max_rows: u32,
    pub hard_timeout: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityVerdict {
    Allow,
    RequireApproval,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum RiskLevel {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityRestrictions {
    pub max_rows: u32,
    pub timeout: Duration,
}

impl SecurityRestrictions {
    pub const fn none() -> Self {
        Self {
            max_rows: 0,
            timeout: Duration::ZERO,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityDecision {
    pub decision_id: Uuid,
    pub verdict: SecurityVerdict,
    pub risk: RiskLevel,
    pub policy_version: String,
    pub restrictions: SecurityRestrictions,
    pub reason_code: &'static str,
}
