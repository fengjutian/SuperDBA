//! Shared helpers for PostgreSQL parity tests.

use std::future::Future;
use std::time::Duration;
use superdba_lib::drivers::postgres;
use superdba_lib::models::{ConnectionParams, DatabaseSelection};
use tokio::time::sleep;

/// Standard connection parameters matching the CI service and local Docker setup.
pub fn pg_params() -> ConnectionParams {
    ConnectionParams {
        driver: "postgres".to_string(),
        host: Some("127.0.0.1".to_string()),
        port: Some(54320),
        username: Some("postgres".to_string()),
        password: Some("password".to_string()),
        database: DatabaseSelection::Single("testdb".to_string()),
        ..Default::default()
    }
}

/// Connection params targeting the secondary database (multi-database tests).
pub fn pg_params_secondary() -> ConnectionParams {
    ConnectionParams {
        database: DatabaseSelection::Single("superdba_test_secondary".to_string()),
        ..pg_params()
    }
}

/// Wait for PostgreSQL to be ready, retrying up to 10 times.
/// Returns `true` if connected, `false` if all retries failed.
pub async fn wait_for_pg() -> bool {
    let params = pg_params();
    for _ in 0..10 {
        if postgres::get_tables(&params, "public").await.is_ok() {
            return true;
        }
        sleep(Duration::from_millis(500)).await;
    }
    false
}

/// Retry a fallible async operation up to `attempts` times when the error looks
/// like a transient pool/connection issue ("connection closed", "pool timed out",
/// "broken pipe"). Non-transient errors are returned immediately.
pub async fn retry_transient<T, E, F, Fut>(attempts: u32, mut f: F) -> Result<T, E>
where
    E: AsRef<str>,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut last_err = None;
    for attempt in 0..attempts {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                let msg = e.as_ref();
                let transient = msg.contains("connection closed")
                    || msg.contains("pool timed out")
                    || msg.contains("broken pipe")
                    || msg.contains("Connection reset");
                if !transient || attempt + 1 == attempts {
                    return Err(e);
                }
                last_err = Some(e);
                sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
            }
        }
    }
    Err(last_err.unwrap())
}

/// Convenience wrapper: retry up to 3 times on transient pool errors.
pub async fn retry<T, F, Fut>(f: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    retry_transient(3, f).await
}
