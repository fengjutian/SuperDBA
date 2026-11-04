//! Shared resource limits for database work initiated by AI surfaces.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const QUERY_TIMEOUT: Duration = Duration::from_secs(30);
pub const METADATA_TIMEOUT: Duration = Duration::from_secs(15);
pub const PREFLIGHT_EXPLAIN_TIMEOUT: Duration = Duration::from_secs(10);
pub const SCHEDULER_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
pub const MCP_DEFAULT_ROWS: u32 = 100;
pub const MCP_MAX_ROWS: u32 = 1_000;
pub const MCP_MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
pub const WORKLOADS_PER_CONNECTION: usize = 2;
pub const GLOBAL_AI_QUERIES: usize = 4;

static GLOBAL_QUERY_SEMAPHORE: once_cell::sync::Lazy<Arc<Semaphore>> =
    once_cell::sync::Lazy::new(|| Arc::new(Semaphore::new(GLOBAL_AI_QUERIES)));

static QUERY_SEMAPHORES: once_cell::sync::Lazy<Mutex<HashMap<String, Weak<Semaphore>>>> =
    once_cell::sync::Lazy::new(Default::default);
static WORKLOAD_SEMAPHORES: once_cell::sync::Lazy<Mutex<HashMap<String, Weak<Semaphore>>>> =
    once_cell::sync::Lazy::new(Default::default);

/// Holds both the process-wide and per-connection capacity for one AI query.
#[derive(Debug)]
pub struct AiQueryPermit {
    _global: OwnedSemaphorePermit,
    _connection: OwnedSemaphorePermit,
}

pub async fn acquire(connection_id: &str) -> Result<AiQueryPermit, String> {
    acquire_ai_permits(
        GLOBAL_QUERY_SEMAPHORE.clone(),
        semaphore(connection_id),
        SCHEDULER_WAIT_TIMEOUT,
    )
    .await
}

async fn acquire_ai_permits(
    global: Arc<Semaphore>,
    connection: Arc<Semaphore>,
    wait: Duration,
) -> Result<AiQueryPermit, String> {
    tokio::time::timeout(wait, async {
        let global = global
            .acquire_owned()
            .await
            .map_err(|_| "The global AI query scheduler is unavailable.".to_string())?;
        let connection = connection
            .acquire_owned()
            .await
            .map_err(|_| "The database query scheduler is unavailable.".to_string())?;
        Ok(AiQueryPermit {
            _global: global,
            _connection: connection,
        })
    })
    .await
    .map_err(|_| "Timed out waiting for AI query capacity.".to_string())?
}

pub async fn acquire_workload(connection_id: &str) -> Result<OwnedSemaphorePermit, String> {
    acquire_with_timeout(
        workload_semaphore(connection_id),
        SCHEDULER_WAIT_TIMEOUT,
        "Timed out waiting for database capacity on this connection.",
    )
    .await
}

async fn acquire_with_timeout(
    semaphore: Arc<Semaphore>,
    wait: Duration,
    timeout_message: &str,
) -> Result<OwnedSemaphorePermit, String> {
    tokio::time::timeout(wait, semaphore.acquire_owned())
        .await
        .map_err(|_| timeout_message.to_string())?
        .map_err(|_| "The database query scheduler is unavailable.".to_string())
}

pub(crate) fn semaphore(connection_id: &str) -> Arc<Semaphore> {
    semaphore_from(&QUERY_SEMAPHORES, connection_id, 1)
}

pub(crate) fn workload_semaphore(connection_id: &str) -> Arc<Semaphore> {
    semaphore_from(
        &WORKLOAD_SEMAPHORES,
        connection_id,
        WORKLOADS_PER_CONNECTION,
    )
}

fn semaphore_from(
    registry: &Mutex<HashMap<String, Weak<Semaphore>>>,
    connection_id: &str,
    permits: usize,
) -> Arc<Semaphore> {
    let mut semaphores = registry.lock().unwrap_or_else(|error| error.into_inner());
    semaphores.retain(|_, semaphore| semaphore.strong_count() > 0);
    if let Some(semaphore) = semaphores.get(connection_id).and_then(Weak::upgrade) {
        return semaphore;
    }
    let semaphore = Arc::new(Semaphore::new(permits));
    semaphores.insert(connection_id.to_string(), Arc::downgrade(&semaphore));
    semaphore
}

#[cfg(test)]
#[path = "ai_query_guard_tests.rs"]
mod tests;
