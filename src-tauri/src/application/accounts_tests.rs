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
