#[cfg(test)]
mod tests {
    use crate::ai_approval::{
        cleanup_expired_in, clear_in, list_pending_in, new_approval_id, poll_decision_in,
        poll_decision_with_liveness_in, read_decision_in, read_pending_in, write_decision_in,
        write_pending_in, ApprovalDecision, PendingApproval, PollOutcome,
    };
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::time::Duration;

    fn make_pending(id: &str) -> PendingApproval {
        PendingApproval {
            id: id.to_string(),
            created_at: "2026-04-24T10:00:00Z".to_string(),
            session_id: "sess-1".to_string(),
            connection_id: "conn-1".to_string(),
            connection_name: "local".to_string(),
            database: None,
            query: "UPDATE orders SET status = 'x'".to_string(),
            query_kind: "write".to_string(),
            query_hash: Some(crate::ai_approval::query_hash(
                "UPDATE orders SET status = 'x'",
            )),
            policy_version: Some("test-policy-v1".to_string()),
            security_decision_id: Some("decision-1".to_string()),
            signature: None,
            client_hint: Some("claude-desktop".to_string()),
            explain_plan: Some(serde_json::json!({"node": "Seq Scan"})),
            explain_error: None,
        }
    }

    fn make_decision(id: &str, decision: &str) -> ApprovalDecision {
        ApprovalDecision {
            approval_id: id.to_string(),
            decided_at: "2026-04-24T10:00:05Z".to_string(),
            decision: decision.to_string(),
            reason: None,
            edited_query: None,
            query_hash: Some(crate::ai_approval::query_hash(
                "UPDATE orders SET status = 'x'",
            )),
            signature: None,
        }
    }

    fn touch(path: &Path) {
        std::fs::write(path, "x").unwrap();
    }

    #[test]
    fn pending_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let p = make_pending("p1");
        write_pending_in(tmp.path(), &p).unwrap();
        let read = read_pending_in(tmp.path(), "p1").unwrap();
        assert_eq!(read.unwrap(), p);
    }

    #[test]
    fn list_pending_returns_only_undecided() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("a")).unwrap();
        write_pending_in(tmp.path(), &make_pending("b")).unwrap();
        // Mark `b` as decided.
        write_decision_in(tmp.path(), &make_decision("b", "approve")).unwrap();
        let list = list_pending_in(tmp.path()).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "a");
    }

    #[test]
    fn clear_in_removes_pending_and_decision() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("a")).unwrap();
        write_decision_in(tmp.path(), &make_decision("a", "deny")).unwrap();
        clear_in(tmp.path(), "a").unwrap();
        assert!(read_pending_in(tmp.path(), "a").unwrap().is_none());
        assert!(read_decision_in(tmp.path(), "a").unwrap().is_none());
        // Clearing an id that never existed is a no-op.
        clear_in(tmp.path(), "missing").unwrap();
    }

    #[test]
    fn list_pending_empty_dir_is_ok() {
        let tmp = TempDir::new().unwrap();
        let list = list_pending_in(tmp.path()).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn list_pending_orders_by_created_at() {
        let tmp = TempDir::new().unwrap();
        let mut a = make_pending("a");
        a.created_at = "2026-04-24T10:00:01Z".into();
        let mut b = make_pending("b");
        b.created_at = "2026-04-24T10:00:00Z".into();
        write_pending_in(tmp.path(), &a).unwrap();
        write_pending_in(tmp.path(), &b).unwrap();
        let list = list_pending_in(tmp.path()).unwrap();
        assert_eq!(list[0].id, "b");
        assert_eq!(list[1].id, "a");
    }

    #[test]
    fn read_decision_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let d = make_decision("xyz", "deny");
        write_pending_in(tmp.path(), &make_pending("xyz")).unwrap();
        write_decision_in(tmp.path(), &d).unwrap();
        assert_eq!(read_decision_in(tmp.path(), "xyz").unwrap(), Some(d));
    }

    #[test]
    fn decision_rejects_tampered_sql_binding() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("bound")).unwrap();
        let mut decision = make_decision("bound", "approve");
        decision.edited_query = Some("DROP TABLE orders".to_string());

        let error = write_decision_in(tmp.path(), &decision).unwrap_err();

        assert!(error.contains("SQL binding is invalid"));
        assert!(read_decision_in(tmp.path(), "bound").unwrap().is_none());
    }

    #[test]
    fn decision_accepts_a_bound_edited_query() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("edited")).unwrap();
        let mut decision = make_decision("edited", "approve");
        decision.edited_query = Some("UPDATE orders SET status = 'approved'".to_string());
        decision.query_hash = Some(crate::ai_approval::query_hash(
            "UPDATE orders SET status = 'approved'",
        ));

        write_decision_in(tmp.path(), &decision).unwrap();

        assert_eq!(
            read_decision_in(tmp.path(), "edited").unwrap(),
            Some(decision)
        );
    }

    #[test]
    fn signed_approval_rejects_tampered_context_or_decision() {
        let key = [7u8; 32];
        let mut pending = make_pending("signed");
        crate::ai_approval::sign_pending(&mut pending, &key).unwrap();
        let mut decision = make_decision("signed", "approve");
        crate::ai_approval::sign_decision(&pending, &mut decision, &key).unwrap();
        crate::ai_approval::verify_signed_decision(&pending, &decision, &key).unwrap();

        let mut tampered_pending = pending.clone();
        tampered_pending.connection_id = "other-connection".to_string();
        assert!(crate::ai_approval::verify_signed_decision(
            &tampered_pending,
            &decision,
            &key
        )
        .is_err());

        let mut tampered_decision = decision.clone();
        tampered_decision.reason = Some("changed after signing".to_string());
        assert!(crate::ai_approval::verify_signed_decision(
            &pending,
            &tampered_decision,
            &key
        )
        .is_err());
    }

    #[test]
    fn consumed_approval_cannot_be_replayed_after_cleanup() {
        let tmp = TempDir::new().unwrap();
        let key = [9u8; 32];
        let mut pending = make_pending("single-use");
        crate::ai_approval::sign_pending(&mut pending, &key).unwrap();
        let mut decision = make_decision("single-use", "approve");
        crate::ai_approval::sign_decision(&pending, &mut decision, &key).unwrap();

        crate::ai_approval::consume_signed_decision_in(
            tmp.path(),
            &pending,
            &decision,
            &key,
        )
        .unwrap();
        clear_in(tmp.path(), "single-use").unwrap();

        let error = crate::ai_approval::consume_signed_decision_in(
            tmp.path(),
            &pending,
            &decision,
            &key,
        )
        .unwrap_err();
        assert!(error.contains("already been consumed"));
    }

    // Polling --------------------------------------------------------------

    #[tokio::test]
    async fn poll_returns_immediate_decision() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("imm")).unwrap();
        write_decision_in(tmp.path(), &make_decision("imm", "approve")).unwrap();
        let res = poll_decision_in(tmp.path(), "imm", 5, 50).await.unwrap();
        assert_eq!(res.unwrap().decision, "approve");
        // Cleanup wipes both files.
        assert!(read_decision_in(tmp.path(), "imm").unwrap().is_none());
        assert!(read_pending_in(tmp.path(), "imm").unwrap().is_none());
    }

    #[tokio::test]
    async fn poll_returns_decision_after_delay() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("late")).unwrap();
        let dir = tmp.path().to_path_buf();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            write_decision_in(&dir, &make_decision("late", "approve")).unwrap();
        });
        let res = poll_decision_in(tmp.path(), "late", 5, 50).await.unwrap();
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn poll_times_out_returning_none() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("never")).unwrap();
        let res = poll_decision_in(tmp.path(), "never", 1, 50).await.unwrap();
        assert!(res.is_none());
        // Pending file should be cleaned up on timeout to avoid leaks.
        assert!(read_pending_in(tmp.path(), "never").unwrap().is_none());
    }

    // Liveness-aware polling ----------------------------------------------

    #[tokio::test]
    async fn liveness_poll_returns_decision_when_alive() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("ok")).unwrap();
        write_decision_in(tmp.path(), &make_decision("ok", "approve")).unwrap();
        let outcome = poll_decision_with_liveness_in(tmp.path(), "ok", 5, 50, || true)
            .await
            .unwrap();
        match outcome {
            PollOutcome::Decided(d) => assert_eq!(d.decision, "approve"),
            other => panic!("expected Decided, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn liveness_poll_exits_early_when_host_dead_before_start() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("dead")).unwrap();
        // 60s timeout, but liveness=false → should return almost instantly.
        let start = std::time::Instant::now();
        let outcome = poll_decision_with_liveness_in(tmp.path(), "dead", 60, 50, || false)
            .await
            .unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(outcome, PollOutcome::HostUnavailable);
        // Pending file should be cleaned up to avoid leaks.
        assert!(read_pending_in(tmp.path(), "dead").unwrap().is_none());
    }

    #[tokio::test]
    async fn liveness_poll_exits_early_when_host_dies_mid_flight() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("flip")).unwrap();
        let alive = Arc::new(AtomicBool::new(true));
        let alive_for_flipper = alive.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            alive_for_flipper.store(false, Ordering::SeqCst);
        });
        let alive_for_check = alive.clone();
        let start = std::time::Instant::now();
        let outcome = poll_decision_with_liveness_in(tmp.path(), "flip", 60, 50, move || {
            alive_for_check.load(Ordering::SeqCst)
        })
        .await
        .unwrap();
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(outcome, PollOutcome::HostUnavailable);
    }

    #[tokio::test]
    async fn liveness_poll_times_out_when_alive_but_no_decision() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("slow")).unwrap();
        let outcome = poll_decision_with_liveness_in(tmp.path(), "slow", 1, 50, || true)
            .await
            .unwrap();
        assert_eq!(outcome, PollOutcome::TimedOut);
    }

    // Concurrency ----------------------------------------------------------

    #[test]
    fn distinct_approval_ids_dont_collide() {
        let tmp = TempDir::new().unwrap();
        let id1 = new_approval_id();
        let id2 = new_approval_id();
        assert_ne!(id1, id2);
        write_pending_in(tmp.path(), &make_pending(&id1)).unwrap();
        write_pending_in(tmp.path(), &make_pending(&id2)).unwrap();
        let list = list_pending_in(tmp.path()).unwrap();
        assert_eq!(list.len(), 2);
    }

    // Target database -------------------------------------------------------

    #[test]
    fn pending_without_database_omits_the_field() {
        let json = serde_json::to_value(make_pending("a")).unwrap();
        assert!(json.get("database").is_none());
    }

    #[test]
    fn pending_round_trips_a_target_database() {
        let tmp = TempDir::new().unwrap();
        let pending = PendingApproval {
            database: Some("analytics".to_string()),
            ..make_pending("db")
        };
        write_pending_in(tmp.path(), &pending).unwrap();
        let read = read_pending_in(tmp.path(), "db").unwrap().unwrap();
        assert_eq!(read.database.as_deref(), Some("analytics"));
    }

    // Cleanup --------------------------------------------------------------

    #[test]
    fn cleanup_removes_old_files_only() {
        let tmp = TempDir::new().unwrap();
        write_pending_in(tmp.path(), &make_pending("recent")).unwrap();
        // Touch a fake old file with stale mtime by lowering it manually.
        let dir = tmp.path().join("pending_approvals");
        let stale_path = dir.join("stale.pending.json");
        touch(&stale_path);
        let old_time = std::time::SystemTime::now()
            .checked_sub(Duration::from_secs(3600))
            .unwrap();
        let _ =
            filetime::set_file_mtime(&stale_path, filetime::FileTime::from_system_time(old_time));
        let deleted = cleanup_expired_in(tmp.path(), 60).unwrap();
        assert_eq!(deleted, 1);
        assert!(!stale_path.exists());
        assert!(read_pending_in(tmp.path(), "recent").unwrap().is_some());
    }
}
