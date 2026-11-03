//! Turning a saved (or requested) [`ConnectionParams`] into the params that
//! actually open a connection: tunnel expansion, connection id for pooling,
//! and the URL-ish credentials a plugin SQL builder may forward.
//!
//! Extracted from the legacy `commands` module so `application/` no longer
//! depends on it. Behaviour is unchanged.

use urlencoding::encode;

use crate::credential_cache;
use crate::models::ConnectionParams;

const DEFAULT_MYSQL_PORT: u16 = 3306;
const DEFAULT_POSTGRES_PORT: u16 = 5432;

pub fn resolve_connection_params(params: &ConnectionParams) -> Result<ConnectionParams, String> {
    crate::application::tunnels::resolve_expanded_connection_params(params, None)
}

/// Resolve params for closing a pool without respawning tunnels.
///
/// `build_connection_key` uses `connection_id` (plus driver/database/TLS), not
/// the tunnel's local port, so closing never needs a live SSH tunnel, SSM
/// session, or kubectl port-forward. Disabling all tunnel types here means a
/// dead bastion / unreachable cluster cannot prevent pool cleanup (#1065).
pub(crate) fn params_for_pool_close(
    params: &ConnectionParams,
    connection_id: &str,
) -> Result<ConnectionParams, String> {
    let mut for_close = params.clone();
    for_close.k8s_enabled = Some(false);
    for_close.ssh_enabled = Some(false);
    for_close.ssm_enabled = Some(false);
    resolve_connection_params_with_id(&for_close, connection_id)
}

/// Resolve connection params and set connection_id for stable pooling
pub fn resolve_connection_params_with_id(
    params: &ConnectionParams,
    connection_id: &str,
) -> Result<ConnectionParams, String> {
    let mut with_id = params.clone();
    with_id.connection_id = Some(connection_id.to_string());
    resolve_connection_params(&with_id)
}

fn runtime_connection_uri(params: &ConnectionParams) -> Option<&str> {
    params
        .connection_uri
        .as_deref()
        .filter(|value| !value.trim().is_empty())
}

/// Re-attach the URI a saved connection needs at runtime, from the session
/// cache or the keychain.
pub(crate) fn restore_runtime_connection_uri(
    cache: &credential_cache::CredentialCache,
    connection_id: &str,
    params: &mut ConnectionParams,
) -> Result<(), String> {
    if runtime_connection_uri(params).is_some() {
        return Ok(());
    }

    let stored_in_keychain = params.connection_uri_in_keychain.unwrap_or(false);
    match credential_cache::get_connection_uri_cached(cache, connection_id, stored_in_keychain) {
        Ok(Some(connection_uri)) => {
            params.connection_uri = Some(connection_uri);
            Ok(())
        }
        Ok(None) if stored_in_keychain => {
            Err("Stored connection URI is unavailable in the OS keychain".to_string())
        }
        Ok(None) => Ok(()),
        Err(_) => Err("Failed to read the stored connection URI from the OS keychain".to_string()),
    }
}

/// Build a `driver://` style URL for display and for drivers that take one.
///
/// Driver implementations build their own URL from capabilities
/// (`DatabaseDriver::build_connection_url`); this is the generic fallback used
/// by the connection form.
pub async fn build_connection_url(params: &ConnectionParams) -> Result<String, String> {
    let user = encode(params.username.as_deref().unwrap_or_default());
    let raw_pass = params.password.as_deref().unwrap_or_default();
    let credentials = if raw_pass.is_empty() {
        user.into_owned()
    } else {
        format!("{}:{}", user, encode(raw_pass))
    };
    let host = params.host.as_deref().unwrap_or("localhost");

    match params.driver.as_str() {
        "sqlite" => Ok(format!("sqlite://{}", params.database)),
        "postgres" => Ok(format!(
            "postgres://{}@{}:{}/{}",
            credentials,
            host,
            params.port.unwrap_or(DEFAULT_POSTGRES_PORT),
            params.database
        )),
        "mysql" => Ok(format!(
            "mysql://{}@{}:{}/{}",
            credentials,
            host,
            params.port.unwrap_or(DEFAULT_MYSQL_PORT),
            params.database
        )),
        _ => Err("Unsupported driver".into()),
    }
}

// --- Test-only shims mirroring the tunnel resolvers -------------------------

/// Check if a string option is empty or contains only whitespace.
#[inline]
#[cfg(test)]
pub(crate) fn is_empty_or_whitespace(s: &Option<String>) -> bool {
    s.as_ref().map(|p| p.trim().is_empty()).unwrap_or(true)
}

#[cfg(test)]
pub(crate) fn resolve_k8s_params(params: &ConnectionParams) -> Result<ConnectionParams, String> {
    crate::application::tunnels::resolve_k8s_params(params)
}

#[cfg(test)]
pub(crate) fn resolve_ssm_params(params: &ConnectionParams) -> Result<ConnectionParams, String> {
    crate::application::tunnels::resolve_ssm_params(params)
}

#[cfg(test)]
use crate::models::SshConnection;

/// Resolves SSH credential (password or passphrase) for testing
/// 1. Credential from request params (if provided, even if empty)
/// 2. Credential from keychain (if save_in_keychain is enabled)
/// 3. Credential from saved connection (as fallback)
#[cfg(test)]
fn resolve_ssh_test_credential(
    request_credential: Option<&str>,
    connection_id: Option<&str>,
    get_ssh_connection: impl Fn(&str) -> Option<SshConnection>,
    get_keychain_credential: impl Fn(&str) -> Result<String, String>,
    extract_saved_credential: impl Fn(&SshConnection) -> Option<String>,
) -> Option<String> {
    // Priority 1: Credential from request
    // If credential field is present in request, use it even if empty
    // Empty string means "use empty credential", not "fallback to keychain"
    if let Some(cred) = request_credential {
        return Some(cred.to_string());
    }

    // If no connection_id, we can't look up saved credentials
    let conn_id = connection_id?;
    let saved = get_ssh_connection(conn_id)?;

    // Priority 2: Credential from keychain
    if saved.save_in_keychain.unwrap_or(false) {
        if let Ok(cred) = get_keychain_credential(conn_id) {
            if !cred.trim().is_empty() {
                return Some(cred);
            }
        }
    }

    // Priority 3: Credential from saved connection
    extract_saved_credential(&saved)
}

/// Fills SSH secrets that are still unresolved from the inline SSH fields of
/// a saved database connection (already hydrated from the keychain by
/// `find_connection_by_id`). Secrets explicitly provided by the request keep
/// priority; blank saved values are ignored.
#[cfg(test)]
pub(crate) fn apply_inline_ssh_secret_fallback(
    resolved_password: Option<String>,
    resolved_passphrase: Option<String>,
    saved_params: &ConnectionParams,
) -> (Option<String>, Option<String>) {
    fn non_blank(value: &Option<String>) -> Option<String> {
        value.as_ref().filter(|v| !v.trim().is_empty()).cloned()
    }
    (
        resolved_password.or_else(|| non_blank(&saved_params.ssh_password)),
        resolved_passphrase.or_else(|| non_blank(&saved_params.ssh_key_passphrase)),
    )
}

/// Helper for backward compatibility - resolves SSH password
#[cfg(test)]
pub(crate) fn resolve_ssh_test_password(
    request_password: Option<&str>,
    connection_id: Option<&str>,
    get_ssh_connection: impl Fn(&str) -> Option<SshConnection>,
    get_keychain_password: impl Fn(&str) -> Result<String, String>,
) -> Option<String> {
    resolve_ssh_test_credential(
        request_password,
        connection_id,
        get_ssh_connection,
        get_keychain_password,
        |conn| {
            conn.password
                .as_ref()
                .filter(|p| !p.trim().is_empty())
                .cloned()
        },
    )
}

#[cfg(test)]
#[path = "connection_params_tests.rs"]
mod tests;
