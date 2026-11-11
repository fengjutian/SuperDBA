use super::plan_facts::{extract, rejection_reason};
use serde_json::json;

#[test]
fn rejects_large_postgres_sequential_scans() {
    let facts = extract(&json!({
        "kind": "plan",
        "plan": {"Node Type": "Seq Scan", "Relation Name": "events", "Plan Rows": 1_500_000}
    }));
    assert_eq!(facts.max_estimated_rows, Some(1_500_000));
    assert!(rejection_reason(&facts).unwrap().contains("full scan"));
}

#[test]
fn rejects_mysql_scan_and_sort_but_allows_small_indexed_reads() {
    let risky = extract(&json!({
        "kind": "raw",
        "raw": {"payload": serde_json::to_string(&json!({
            "table": {"table_name": "orders", "access_type": "ALL", "rows_examined_per_scan": 250_000},
            "ordering_operation": {"using_filesort": true}
        })).unwrap()}
    }));
    assert!(rejection_reason(&risky).unwrap().contains("scan and sort"));

    let safe = extract(&json!({
        "kind": "plan",
        "plan": {"Node Type": "Index Scan", "Relation Name": "orders", "Plan Rows": 25}
    }));
    assert_eq!(rejection_reason(&safe), None);
}

#[test]
fn recognizes_sqlite_text_nested_inside_a_json_payload() {
    let facts = extract(&json!({
        "kind": "raw",
        "raw": {"payload": serde_json::to_string(&json!([[0, 0, 0, "SCAN audit_log"]])).unwrap()}
    }));
    assert_eq!(facts.full_scans, vec!["audit_log (SCAN)"]);
    assert_eq!(rejection_reason(&facts), None);
}

#[test]
fn rejects_unrecognized_plans() {
    let facts = extract(&json!({"kind": "raw", "raw": {"payload": ""}}));
    assert!(rejection_reason(&facts).unwrap().contains("could not be evaluated"));
}
