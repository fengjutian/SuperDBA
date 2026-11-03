use std::panic::{catch_unwind, AssertUnwindSafe};

use super::credential_cache::{CacheEntry, CredentialCache};
use crate::sync_ext::lock;

/// A panic while a cache guard is held must not turn every later credential
/// lookup into a panic: the maps only ever hold plain `HashMap` state, so the
/// poison carries no corrupt data and the next lock recovers it.
#[test]
fn a_poisoned_map_stays_usable() {
    let cache = CredentialCache::default();

    let poisoned = catch_unwind(AssertUnwindSafe(|| {
        let _guard = cache.db_passwords.lock().unwrap();
        panic!("boom");
    }));
    assert!(poisoned.is_err());
    assert!(cache.db_passwords.is_poisoned());

    lock(&cache.db_passwords).insert("conn-1".to_string(), CacheEntry::Absent);
    assert!(matches!(
        lock(&cache.db_passwords).get("conn-1"),
        Some(CacheEntry::Absent)
    ));
}

/// The recovery helper must keep the ordinary, unpoisoned path intact.
#[test]
fn an_unpoisoned_map_round_trips() {
    let cache = CredentialCache::default();

    lock(&cache.ai_keys).insert(
        "provider".to_string(),
        CacheEntry::Present("key".to_string()),
    );

    assert!(matches!(
        lock(&cache.ai_keys).get("provider"),
        Some(CacheEntry::Present(value)) if value == "key"
    ));
}

/// Every cache map is independent: locking one must never contend with another.
#[test]
fn maps_are_independent() {
    let cache = CredentialCache::default();

    let _held = lock(&cache.db_passwords);
    lock(&cache.ssh_passwords).insert("ssh".to_string(), CacheEntry::Absent);

    assert!(lock(&cache.ssh_passwords).contains_key("ssh"));
}
