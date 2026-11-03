//! Fixtures shared by the `application` unit tests.
//!
//! Test-only: declared `#[cfg(test)]` from `application/mod.rs`, so nothing
//! here is linked into a production build.

use crate::models::{ConnectionGroup, ConnectionParams, DatabaseSelection, SavedConnection};

/// A plain MySQL connection pointing at `testdb`.
pub(crate) fn base_params() -> ConnectionParams {
    ConnectionParams {
        driver: "mysql".to_string(),
        host: Some("localhost".to_string()),
        port: Some(3306),
        username: Some("root".to_string()),
        database: DatabaseSelection::Single("testdb".to_string()),
        ..Default::default()
    }
}

/// A saved connection whose params derive from [`base_params`].
pub(crate) fn saved_connection(
    id: &str,
    password: Option<&str>,
    save_in_keychain: bool,
) -> SavedConnection {
    saved_connection_with_params(
        ConnectionParams {
            password: password.map(|p| p.to_string()),
            save_in_keychain: Some(save_in_keychain),
            ..base_params()
        },
        id,
    )
}

/// A saved connection wrapping `params`, named after its id.
pub(crate) fn saved_connection_with_params(
    params: ConnectionParams,
    id: &str,
) -> SavedConnection {
    SavedConnection {
        id: id.to_string(),
        name: "Test".to_string(),
        params,
        plugin_secret_keys: Vec::new(),
        group_id: None,
        sort_order: None,
        detect_json_in_text_columns: None,
        appearance: None,
        tag_ids: None,
        environment: None,
    }
}

/// A group whose name matches its id.
pub(crate) fn group(id: &str, parent: Option<&str>) -> ConnectionGroup {
    ConnectionGroup {
        id: id.to_string(),
        name: id.to_string(),
        collapsed: false,
        sort_order: 0,
        parent_id: parent.map(|p| p.to_string()),
    }
}

/// A saved connection placed directly in `group_id`.
pub(crate) fn connection_in_group(id: &str, group_id: Option<&str>) -> SavedConnection {
    let mut connection = saved_connection(id, None, false);
    connection.group_id = group_id.map(|g| g.to_string());
    connection
}
