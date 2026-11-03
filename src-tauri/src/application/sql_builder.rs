//! Params handed to the plugin SQL builders.
//!
//! A builder only renders SQL: it must never open a tunnel, and it must work
//! while the server is unreachable. These helpers therefore apply the
//! browsed database and the keychain secrets, and nothing else.
//!
//! Extracted from the legacy `commands` module so `application/` no longer
//! depends on it. Behaviour is unchanged.

use crate::models::{ConnectionParams, SavedConnection};

fn runtime_connection_uri(params: &ConnectionParams) -> Option<&str> {
    params
        .connection_uri
        .as_deref()
        .filter(|value| !value.trim().is_empty())
}

/// Point `params` at the database the user is browsing (multi-database
/// layouts, #822). An empty or missing override keeps the saved selection.
pub(crate) fn apply_ddl_database_override(params: &mut ConnectionParams, database: Option<String>) {
    if let Some(db) = database.filter(|d| !d.is_empty()) {
        params.database = crate::models::DatabaseSelection::Single(db);
    }
}

/// Bind the keychain secrets the SQL builders may forward to a plugin (the
/// connection URI and the database password), without ever failing: a
/// builder only renders SQL, so a keychain error leaves the secret unset and
/// the params are bound without it. SSH secrets are not loaded because the
/// builders never open tunnels.
pub(crate) fn apply_sql_builder_secrets(
    params: &mut ConnectionParams,
    connection_id: &str,
    connection_uri: impl FnOnce(bool) -> Result<Option<String>, String>,
    db_password: impl FnOnce() -> Result<String, String>,
) {
    if runtime_connection_uri(params).is_none() {
        let stored_in_keychain = params.connection_uri_in_keychain.unwrap_or(false);
        match connection_uri(stored_in_keychain) {
            Ok(Some(uri)) => params.connection_uri = Some(uri),
            Ok(None) if stored_in_keychain => log::warn!(
                "Connection URI for {connection_id} is missing from the OS keychain; building SQL without it"
            ),
            Ok(None) => {}
            Err(e) => log::warn!(
                "Failed to read the connection URI for {connection_id} from the OS keychain; building SQL without it: {e}"
            ),
        }
    }
    if params.save_in_keychain.unwrap_or(false) && !params.use_iam_auth.unwrap_or(false) {
        match db_password() {
            Ok(password) => params.password = Some(password),
            Err(e) => log::warn!(
                "Failed to read the DB password for {connection_id} from the OS keychain; building SQL without it: {e}"
            ),
        }
    }
}

/// Params forwarded to the plugin SQL builders: the saved params with the
/// connection id and the browsed `database` applied. SSH/SSM/K8s tunnels and
/// proxies are never resolved here, so host and port stay as saved: a
/// preview must not open a tunnel, and must work while the server is
/// unreachable.
pub(crate) fn sql_builder_params(
    saved: &SavedConnection,
    database: Option<String>,
) -> ConnectionParams {
    let mut params = saved.params.clone();
    params.connection_id = Some(saved.id.clone());
    if crate::drivers::registry::is_local_path_driver(&params.driver) {
        params.database = crate::fs_path::sanitize_database_selection(&params.database);
    }
    apply_ddl_database_override(&mut params, database);
    params
}
