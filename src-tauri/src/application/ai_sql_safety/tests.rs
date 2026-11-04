use super::{
    validate_ai_sql, LOCK_HEAVY_MESSAGE, TRANSACTION_CONTROL_MESSAGE, UNBOUNDED_WRITE_MESSAGE,
};

#[test]
fn allows_bounded_updates_and_deletes() {
    assert_eq!(
        validate_ai_sql("UPDATE users SET active = 0 WHERE id = 7"),
        Ok(())
    );
    assert_eq!(
        validate_ai_sql("DELETE FROM sessions WHERE expires_at < NOW()"),
        Ok(())
    );
}

#[test]
fn blocks_updates_and_deletes_without_where() {
    assert_eq!(
        validate_ai_sql("UPDATE users SET active = 0"),
        Err(UNBOUNDED_WRITE_MESSAGE)
    );
    assert_eq!(
        validate_ai_sql("DELETE FROM sessions"),
        Err(UNBOUNDED_WRITE_MESSAGE)
    );
}

#[test]
fn checks_where_at_the_same_nesting_depth() {
    let unsafe_outer_update =
        "WITH ids AS (SELECT id FROM users WHERE stale = 1) UPDATE users SET active = 0";
    assert_eq!(
        validate_ai_sql(unsafe_outer_update),
        Err(UNBOUNDED_WRITE_MESSAGE)
    );

    let safe_outer_update =
        "WITH ids AS (SELECT id FROM users) UPDATE users SET active = 0 WHERE id IN (SELECT id FROM ids)";
    assert_eq!(validate_ai_sql(safe_outer_update), Ok(()));
}

#[test]
fn ignores_keywords_inside_strings_comments_and_quoted_identifiers() {
    let sql = "SELECT 'DELETE FROM users', \"ALTER\" FROM reports -- UPDATE users\nWHERE id = 1";
    assert_eq!(validate_ai_sql(sql), Ok(()));
}

#[test]
fn blocks_lock_heavy_ddl_and_maintenance() {
    for sql in [
        "ALTER TABLE users ADD COLUMN note TEXT",
        "TRUNCATE TABLE audit_log",
        "CREATE INDEX idx_users_email ON users(email)",
        "CREATE UNIQUE INDEX idx_users_email ON users(email)",
        "DROP INDEX idx_users_email",
        "VACUUM FULL users",
        "OPTIMIZE TABLE users",
        "REINDEX TABLE users",
        "CLUSTER users USING idx_users_email",
        "LOCK TABLE users IN ACCESS EXCLUSIVE MODE",
    ] {
        assert_eq!(validate_ai_sql(sql), Err(LOCK_HEAVY_MESSAGE), "{sql}");
    }
}

#[test]
fn blocks_transaction_control() {
    for sql in [
        "BEGIN",
        "START TRANSACTION READ ONLY",
        "COMMIT",
        "END TRANSACTION",
        "ROLLBACK TO SAVEPOINT before_update",
        "SAVEPOINT before_update",
        "RELEASE SAVEPOINT before_update",
        "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
        "SET SESSION TRANSACTION READ ONLY",
        "SET autocommit = 0",
    ] {
        assert_eq!(
            validate_ai_sql(sql),
            Err(TRANSACTION_CONTROL_MESSAGE),
            "{sql}"
        );
    }
}

#[test]
fn does_not_treat_read_only_text_as_an_operation() {
    assert_eq!(
        validate_ai_sql("SELECT operation FROM jobs WHERE operation = 'VACUUM'"),
        Ok(())
    );
    assert_eq!(
        validate_ai_sql("SELECT cluster, vacuum, optimize FROM job_metrics"),
        Ok(())
    );
    assert_eq!(
        validate_ai_sql("SELECT 'BEGIN', 'COMMIT', autocommit FROM settings"),
        Ok(())
    );
}
