use super::accounts::AccountStore;

#[test]
fn bootstraps_and_authenticates_a_named_account() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    let id = store
        .bootstrap_administrator("alice@example.com", "correct horse battery staple")
        .unwrap();

    let principal = store
        .authenticate("alice@example.com", "correct horse battery staple")
        .unwrap();
    assert_eq!(principal.id, id);
    assert_eq!(principal.subject, "alice@example.com");
    assert!(store
        .authenticate("alice@example.com", "incorrect password")
        .is_none());

    let reopened = AccountStore::open(directory.path()).unwrap();
    assert!(reopened
        .authenticate("alice@example.com", "correct horse battery staple")
        .is_some());
}

#[test]
fn refuses_a_second_bootstrap_account() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    store
        .bootstrap_administrator("first", "correct horse battery staple")
        .unwrap();
    let error = store
        .bootstrap_administrator("second", "another correct horse battery staple")
        .unwrap_err();
    assert!(error.contains("already been initialized"));
}

#[test]
fn rejects_weak_bootstrap_credentials() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    assert!(store
        .bootstrap_administrator("", "long enough password")
        .is_err());
    assert!(store.bootstrap_administrator("alice", "short").is_err());
    assert!(!store.has_accounts());
}

#[test]
fn creates_disables_and_reenables_accounts() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    store
        .bootstrap_administrator("admin", "correct horse battery staple")
        .unwrap();
    let operator = store
        .create("operator", "another correct horse battery staple")
        .unwrap();
    assert_eq!(store.list().len(), 2);

    let disabled = store.set_disabled(operator.id, true).unwrap();
    assert!(disabled.disabled);
    assert!(store
        .authenticate("operator", "another correct horse battery staple")
        .is_none());

    store.set_disabled(operator.id, false).unwrap();
    assert!(store
        .authenticate("operator", "another correct horse battery staple")
        .is_some());
}

#[test]
fn changes_password_and_rejects_duplicate_usernames() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    let account_id = store
        .bootstrap_administrator("admin", "correct horse battery staple")
        .unwrap();
    assert!(store
        .create("admin", "another correct horse battery staple")
        .is_err());

    store
        .change_password(account_id, "replacement horse battery staple")
        .unwrap();
    assert!(store
        .authenticate("admin", "correct horse battery staple")
        .is_none());
    assert!(store
        .authenticate("admin", "replacement horse battery staple")
        .is_some());
}

#[test]
fn keeps_at_least_one_active_account() {
    let directory = tempfile::tempdir().unwrap();
    let store = AccountStore::open(directory.path()).unwrap();
    let account_id = store
        .bootstrap_administrator("admin", "correct horse battery staple")
        .unwrap();

    let error = store.set_disabled(account_id, true).unwrap_err();
    assert!(error.contains("last active"));
    assert!(store
        .authenticate("admin", "correct horse battery staple")
        .is_some());
}
