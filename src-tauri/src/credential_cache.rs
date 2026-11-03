use std::collections::HashMap;
use std::sync::Mutex;

use crate::sync_ext::lock;

/// Cached credential value.
/// `Present` = keychain returned a value.
/// `Absent`  = keychain confirmed no entry exists (NoEntry). Caching misses
///              avoids redundant IPC round-trips on every DB command.
#[derive(Clone, Debug)]
pub enum CacheEntry {
    Present(String),
    Absent,
}

/// In-memory credential cache backed by separate maps per credential type.
///
/// Uses `std::sync::Mutex` (not `tokio::sync::Mutex`) because all critical
/// sections are pure HashMap operations (nanoseconds) and are never held
/// across `.await` points. Wrapping the struct in `Arc` lets callers clone
/// it into `tokio::task::spawn_blocking` closures.
pub struct CredentialCache {
    pub db_passwords: Mutex<HashMap<String, CacheEntry>>,
    pub connection_uris: Mutex<HashMap<String, CacheEntry>>,
    pub plugin_secrets: Mutex<HashMap<String, CacheEntry>>,
    pub ssh_passwords: Mutex<HashMap<String, CacheEntry>>,
    pub ssh_passphrases: Mutex<HashMap<String, CacheEntry>>,
    pub ai_keys: Mutex<HashMap<String, CacheEntry>>,
}

impl Default for CredentialCache {
    fn default() -> Self {
        Self {
            db_passwords: Mutex::new(HashMap::new()),
            connection_uris: Mutex::new(HashMap::new()),
            plugin_secrets: Mutex::new(HashMap::new()),
            ssh_passwords: Mutex::new(HashMap::new()),
            ssh_passphrases: Mutex::new(HashMap::new()),
            ai_keys: Mutex::new(HashMap::new()),
        }
    }
}

pub(crate) fn plugin_secret_cache_key(connection_id: &str, driver: &str, key: &str) -> String {
    format!("{}\0{}\0{}", connection_id, driver, key)
}

// ─── Read-through helpers ─────────────────────────────────────────────────────
// These functions are synchronous and intended to be called from inside
// `tokio::task::spawn_blocking` when used in async contexts.

/// Get DB password: check cache first, fall through to keychain on miss.
pub fn get_db_password_cached(
    cache: &CredentialCache,
    connection_id: &str,
) -> Result<String, String> {
    {
        let guard = lock(&cache.db_passwords);
        match guard.get(connection_id) {
            Some(CacheEntry::Present(v)) => return Ok(v.clone()),
            Some(CacheEntry::Absent) => return Err("No entry".to_string()),
            None => {}
        }
    }
    let result = crate::keychain_utils::get_db_password(connection_id, "");
    {
        let mut guard = lock(&cache.db_passwords);
        guard.insert(
            connection_id.to_string(),
            match &result {
                Ok(v) => CacheEntry::Present(v.clone()),
                Err(_) => CacheEntry::Absent,
            },
        );
    }
    result
}

/// Read a connection URI from this process first. Only consult the OS keychain
/// when the persisted marker explicitly says an entry was written for it.
pub fn get_connection_uri_cached(
    cache: &CredentialCache,
    connection_id: &str,
    load_from_keychain: bool,
) -> Result<Option<String>, String> {
    {
        let guard = lock(&cache.connection_uris);
        match guard.get(connection_id) {
            Some(CacheEntry::Present(v)) => return Ok(Some(v.clone())),
            Some(CacheEntry::Absent) => return Ok(None),
            None => {}
        }
    }
    if !load_from_keychain {
        return Ok(None);
    }

    let value = crate::keychain_utils::get_connection_uri(connection_id)?;
    lock(&cache.connection_uris).insert(
        connection_id.to_string(),
        CacheEntry::Present(value.clone()),
    );
    Ok(Some(value))
}

pub fn get_plugin_secret_cached(
    cache: &CredentialCache,
    connection_id: &str,
    driver: &str,
    key: &str,
) -> Result<Option<String>, String> {
    let cache_key = plugin_secret_cache_key(connection_id, driver, key);
    {
        let guard = lock(&cache.plugin_secrets);
        match guard.get(&cache_key) {
            Some(CacheEntry::Present(value)) => return Ok(Some(value.clone())),
            Some(CacheEntry::Absent) => return Ok(None),
            None => {}
        }
    }

    let value = crate::keychain_utils::get_plugin_secret(connection_id, driver, key)?;
    lock(&cache.plugin_secrets).insert(
        cache_key,
        match &value {
            Some(value) => CacheEntry::Present(value.clone()),
            None => CacheEntry::Absent,
        },
    );
    Ok(value)
}

/// Get SSH password: check cache first, fall through to keychain on miss.
pub fn get_ssh_password_cached(
    cache: &CredentialCache,
    connection_id: &str,
) -> Result<String, String> {
    {
        let guard = lock(&cache.ssh_passwords);
        match guard.get(connection_id) {
            Some(CacheEntry::Present(v)) => return Ok(v.clone()),
            Some(CacheEntry::Absent) => return Err("No entry".to_string()),
            None => {}
        }
    }
    let result = crate::keychain_utils::get_ssh_password(connection_id, "");
    {
        let mut guard = lock(&cache.ssh_passwords);
        guard.insert(
            connection_id.to_string(),
            match &result {
                Ok(v) => CacheEntry::Present(v.clone()),
                Err(_) => CacheEntry::Absent,
            },
        );
    }
    result
}

/// Get SSH key passphrase: check cache first, fall through to keychain on miss.
pub fn get_ssh_key_passphrase_cached(
    cache: &CredentialCache,
    connection_id: &str,
) -> Result<String, String> {
    {
        let guard = lock(&cache.ssh_passphrases);
        match guard.get(connection_id) {
            Some(CacheEntry::Present(v)) => return Ok(v.clone()),
            Some(CacheEntry::Absent) => return Err("No entry".to_string()),
            None => {}
        }
    }
    let result = crate::keychain_utils::get_ssh_key_passphrase(connection_id, "");
    {
        let mut guard = lock(&cache.ssh_passphrases);
        guard.insert(
            connection_id.to_string(),
            match &result {
                Ok(v) => CacheEntry::Present(v.clone()),
                Err(_) => CacheEntry::Absent,
            },
        );
    }
    result
}

/// Get AI API key: check cache first, fall through to keychain on miss.
pub fn get_ai_key_cached(cache: &CredentialCache, provider: &str) -> Result<String, String> {
    {
        let guard = lock(&cache.ai_keys);
        match guard.get(provider) {
            Some(CacheEntry::Present(v)) => return Ok(v.clone()),
            Some(CacheEntry::Absent) => return Err("No entry".to_string()),
            None => {}
        }
    }
    match crate::keychain_utils::get_ai_key(provider) {
        // A value, or a definitive miss (NoEntry): both are safe to memoize so
        // we never re-prompt for this provider again this session.
        Ok(maybe) => {
            let entry = match &maybe {
                Some(v) => CacheEntry::Present(v.clone()),
                None => CacheEntry::Absent,
            };
            lock(&cache.ai_keys).insert(provider.to_string(), entry);
            maybe.ok_or_else(|| "No entry".to_string())
        }
        // Transient failure (denied prompt, timeout, securityd error): do NOT
        // cache, so the next read retries the keychain instead of pinning the
        // key as permanently absent.
        Err(e) => Err(e),
    }
}

// ─── Write-through helpers ────────────────────────────────────────────────────
// Call these AFTER the corresponding keychain_utils::set_* succeeds.

pub fn set_db_password_cached(cache: &CredentialCache, connection_id: &str, password: &str) {
    lock(&cache.db_passwords).insert(
        connection_id.to_string(),
        CacheEntry::Present(password.to_string()),
    );
}

pub fn set_connection_uri_cached(
    cache: &CredentialCache,
    connection_id: &str,
    connection_uri: &str,
) {
    lock(&cache.connection_uris).insert(
        connection_id.to_string(),
        CacheEntry::Present(connection_uri.to_string()),
    );
}

pub fn set_plugin_secret_cached(
    cache: &CredentialCache,
    connection_id: &str,
    driver: &str,
    key: &str,
    value: &str,
) {
    lock(&cache.plugin_secrets).insert(
        plugin_secret_cache_key(connection_id, driver, key),
        CacheEntry::Present(value.to_string()),
    );
}

pub fn set_ssh_password_cached(cache: &CredentialCache, connection_id: &str, password: &str) {
    lock(&cache.ssh_passwords).insert(
        connection_id.to_string(),
        CacheEntry::Present(password.to_string()),
    );
}

pub fn set_ssh_key_passphrase_cached(
    cache: &CredentialCache,
    connection_id: &str,
    passphrase: &str,
) {
    lock(&cache.ssh_passphrases).insert(
        connection_id.to_string(),
        CacheEntry::Present(passphrase.to_string()),
    );
}

pub fn set_ai_key_cached(cache: &CredentialCache, provider: &str, key: &str) {
    cache
        .ai_keys
        .lock()
        .unwrap()
        .insert(provider.to_string(), CacheEntry::Present(key.to_string()));
}

// ─── Invalidation helpers ─────────────────────────────────────────────────────
// Call these AFTER the corresponding keychain_utils::delete_* succeeds.
// Removing the entry forces the next read to re-query the keychain (which will
// return NoEntry, caching Absent).

pub fn invalidate_db_password(cache: &CredentialCache, connection_id: &str) {
    lock(&cache.db_passwords).remove(connection_id);
}

pub fn invalidate_connection_uri(cache: &CredentialCache, connection_id: &str) {
    lock(&cache.connection_uris).remove(connection_id);
}

pub fn invalidate_plugin_secret(
    cache: &CredentialCache,
    connection_id: &str,
    driver: &str,
    key: &str,
) {
    cache
        .plugin_secrets
        .lock()
        .unwrap()
        .remove(&plugin_secret_cache_key(connection_id, driver, key));
}

pub fn invalidate_ssh_password(cache: &CredentialCache, connection_id: &str) {
    lock(&cache.ssh_passwords).remove(connection_id);
}

pub fn invalidate_ssh_key_passphrase(cache: &CredentialCache, connection_id: &str) {
    lock(&cache.ssh_passphrases).remove(connection_id);
}

pub fn invalidate_ai_key(cache: &CredentialCache, provider: &str) {
    lock(&cache.ai_keys).remove(provider);
}

/// Invalidate all cached credentials for a connection ID (e.g. on delete).
pub fn invalidate_all_for_connection(cache: &CredentialCache, connection_id: &str) {
    lock(&cache.db_passwords).remove(connection_id);
    lock(&cache.connection_uris).remove(connection_id);
    lock(&cache.ssh_passwords).remove(connection_id);
    lock(&cache.ssh_passphrases).remove(connection_id);
    let prefix = format!("{}\0", connection_id);
    cache
        .plugin_secrets
        .lock()
        .unwrap()
        .retain(|key, _| !key.starts_with(&prefix));
}
