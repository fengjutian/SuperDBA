use crate::ui_state::{
    close, database_path, delete_entry, encode_value, get_entries, set_entry, validate_key,
    MAX_KEYS_PER_READ, MAX_KEY_LENGTH, MAX_VALUE_BYTES,
};
use serde_json::json;

#[test]
fn accepts_existing_frontend_keys() {
    for key in [
        "superdba_sidebar_width",
        "superdba_row_editor_sidebar_width",
        "superdba_last_seen_version",
        "superdba_support_prompt_dismissed",
        "superdba:discord-callout-v2-dismissed",
        "superdba.format.v1",
    ] {
        assert!(validate_key(key).is_ok(), "{key}");
    }
}

#[test]
fn rejects_malformed_keys() {
    for key in [
        String::new(),
        "Upper".to_string(),
        "with space".to_string(),
        "../escape".to_string(),
        "k".repeat(MAX_KEY_LENGTH + 1),
    ] {
        assert!(validate_key(&key).is_err(), "{key}");
    }
}

#[test]
fn caps_value_size() {
    assert!(encode_value(&json!("x".repeat(MAX_VALUE_BYTES - 2))).is_ok());
    assert!(encode_value(&json!("x".repeat(MAX_VALUE_BYTES))).is_err());
}

#[tokio::test]
async fn stores_reads_and_deletes_entries() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());

    assert!(get_entries(&database, None).await.unwrap().is_empty());
    set_entry(&database, "superdba_sidebar_width", &json!(320))
        .await
        .unwrap();
    set_entry(
        &database,
        "superdba_support_prompt_dismissed",
        &json!(true),
    )
    .await
    .unwrap();
    set_entry(&database, "superdba_sidebar_width", &json!(280))
        .await
        .unwrap();

    let all = get_entries(&database, None).await.unwrap();
    assert_eq!(all.get("superdba_sidebar_width"), Some(&json!(280)));
    assert_eq!(
        all.get("superdba_support_prompt_dismissed"),
        Some(&json!(true))
    );

    let selected = get_entries(
        &database,
        Some(&["superdba_sidebar_width".to_string(), "missing".to_string()]),
    )
    .await
    .unwrap();
    assert_eq!(selected.len(), 1);

    assert!(delete_entry(&database, "superdba_sidebar_width")
        .await
        .unwrap());
    assert!(!delete_entry(&database, "superdba_sidebar_width")
        .await
        .unwrap());
    assert_eq!(get_entries(&database, None).await.unwrap().len(), 1);
    close(&database).await;
}

#[tokio::test]
async fn persists_across_reopened_pools() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());
    set_entry(&database, "superdba_last_seen_version", &json!("0.25.0"))
        .await
        .unwrap();
    close(&database).await;

    let entries = get_entries(&database, None).await.unwrap();
    assert_eq!(
        entries.get("superdba_last_seen_version"),
        Some(&json!("0.25.0"))
    );
    close(&database).await;
}

#[tokio::test]
async fn returns_only_the_requested_keys() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());
    for (key, value) in [
        ("superdba_sidebar_width", json!(320)),
        ("superdba_support_prompt_dismissed", json!(true)),
        ("superdba_last_seen_version", json!("0.25.0")),
    ] {
        set_entry(&database, key, &value).await.unwrap();
    }

    let selected = get_entries(
        &database,
        Some(&[
            "superdba_sidebar_width".to_string(),
            // Duplicates and unknown keys must not disturb the result.
            "superdba_sidebar_width".to_string(),
            "missing_key".to_string(),
        ]),
    )
    .await
    .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected.get("superdba_sidebar_width"), Some(&json!(320)));

    close(&database).await;
}

#[tokio::test]
async fn empty_key_list_short_circuits_without_opening_the_database() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());

    let entries = get_entries(&database, Some(&[])).await.unwrap();
    assert!(entries.is_empty());
    assert!(!database.exists());

    close(&database).await;
}

#[tokio::test]
async fn concurrent_first_reads_share_one_pool() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());
    set_entry(&database, "superdba_sidebar_width", &json!(300))
        .await
        .unwrap();
    close(&database).await;

    // Every task races to open the database after the pool was dropped; all of
    // them must succeed and observe the same stored value.
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let database = database.clone();
        tasks.push(tokio::spawn(async move {
            get_entries(&database, Some(&["superdba_sidebar_width".to_string()])).await
        }));
    }
    for task in tasks {
        let entries = task.await.unwrap().unwrap();
        assert_eq!(entries.get("superdba_sidebar_width"), Some(&json!(300)));
    }

    close(&database).await;
}

#[tokio::test]
async fn opens_in_normal_synchronous_mode() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());

    let pool = crate::ui_state::pool(&database).await.unwrap();
    // 0 = OFF, 1 = NORMAL, 2 = FULL, 3 = EXTRA. WAL pairs with NORMAL so a
    // commit does not fsync, which is what makes the frequent small writes cheap.
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(synchronous, 1);

    close(&database).await;
}

#[tokio::test]
async fn rejects_invalid_requests_before_touching_the_database() {
    let temp = tempfile::tempdir().unwrap();
    let database = database_path(temp.path());
    assert!(set_entry(&database, "Bad Key", &json!(1)).await.is_err());
    let too_many: Vec<String> = (0..=MAX_KEYS_PER_READ).map(|i| format!("k{i}")).collect();
    assert!(get_entries(&database, Some(&too_many)).await.is_err());
    assert!(!database.exists());
}
