use super::{
    validate_ai_sql, LOCKING_READ_MESSAGE, LOCK_HEAVY_MESSAGE, MAX_AI_SQL_BYTES,
    RESOURCE_ABUSE_MESSAGE, SQL_TOO_LARGE_MESSAGE, TRANSACTION_CONTROL_MESSAGE,
    UNBOUNDED_WRITE_MESSAGE,
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
fn blocks_locking_reads_and_advisory_locks() {
    for sql in [
        "SELECT * FROM jobs WHERE id = 7 FOR UPDATE",
        "SELECT * FROM jobs FOR NO KEY UPDATE",
        "SELECT * FROM jobs FOR SHARE",
        "SELECT * FROM jobs FOR KEY SHARE",
        "SELECT * FROM jobs LOCK IN SHARE MODE",
        "SELECT pg_advisory_lock(42)",
        "SELECT pg_try_advisory_xact_lock(42)",
        "SELECT GET_LOCK('maintenance', 10)",
        "EXEC sp_getapplock @Resource = 'maintenance'",
    ] {
        assert_eq!(validate_ai_sql(sql), Err(LOCKING_READ_MESSAGE), "{sql}");
    }
}

#[test]
fn blocks_oversized_sql_before_tokenizing_it() {
    let sql = "x".repeat(MAX_AI_SQL_BYTES + 1);
    assert_eq!(validate_ai_sql(&sql), Err(SQL_TOO_LARGE_MESSAGE));
}

#[test]
fn blocks_delay_and_benchmark_operations() {
    for sql in [
        "SELECT pg_sleep(30)",
        "SELECT pg_sleep_for('30 seconds')",
        "SELECT pg_sleep_until(NOW() + INTERVAL '30 seconds')",
        "SELECT SLEEP(30)",
        "SELECT BENCHMARK(1000000, SHA2('value', 256))",
        "WAITFOR DELAY '00:00:30'",
        "WAITFOR TIME '23:59:59'",
    ] {
        assert_eq!(validate_ai_sql(sql), Err(RESOURCE_ABUSE_MESSAGE), "{sql}");
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
    assert_eq!(
        validate_ai_sql("SELECT 'FOR UPDATE', 'GET_LOCK', advisory_lock_count FROM metrics"),
        Ok(())
    );
    assert_eq!(
        validate_ai_sql("SELECT sleep_count, benchmark_name, 'pg_sleep(30)' FROM metrics"),
        Ok(())
    );
}
