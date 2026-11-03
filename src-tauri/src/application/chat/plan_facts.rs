//! Heuristic facts extracted from dialect-specific EXPLAIN output.
//!
//! Every driver returns a different plan shape, so the chat diagnosis tool does
//! not try to model them. It walks the JSON (or scans raw text) for the handful
//! of signals that drive an answer: which relations appear, whether the plan
//! starts with a sequential/table scan, whether any index is used, and whether
//! the plan carries row estimates.

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanFacts {
    pub node_count: usize,
    pub relations: Vec<String>,
    pub full_scans: Vec<String>,
    pub uses_index: bool,
    pub has_row_estimates: bool,
    pub hints: Vec<String>,
}

/// Extracts facts from the serialized `ExplainQueryOutput`.
pub fn extract(output: &Value) -> PlanFacts {
    let mut facts = PlanFacts::default();
    let mut relations = BTreeSet::new();
    let mut full_scans = BTreeSet::new();

    match output.get("kind").and_then(Value::as_str) {
        Some("plan") => {
            if let Some(plan) = output.get("plan") {
                walk(plan, &mut facts, &mut relations, &mut full_scans, 0);
            }
        }
        Some("raw") => {
            let payload = output
                .pointer("/raw/payload")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !payload.trim().is_empty() {
                match serde_json::from_str::<Value>(payload) {
                    Ok(parsed) => walk(&parsed, &mut facts, &mut relations, &mut full_scans, 0),
                    Err(_) => scan_text(payload, &mut facts, &mut relations, &mut full_scans),
                }
            }
        }
        _ => {}
    }

    facts.relations = relations.into_iter().collect();
    facts.full_scans = full_scans.into_iter().collect();
    facts.hints = build_hints(&facts);
    facts
}

const MAX_DEPTH: usize = 32;

fn walk(
    value: &Value,
    facts: &mut PlanFacts,
    relations: &mut BTreeSet<String>,
    full_scans: &mut BTreeSet<String>,
    depth: usize,
) {
    if depth > MAX_DEPTH {
        return;
    }
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, facts, relations, full_scans, depth + 1);
            }
        }
        Value::Object(map) => {
            let node_type = string_at(map, "Node Type");
            let relation = string_at(map, "Relation Name")
                .or_else(|| string_at(map, "table_name"))
                .or_else(|| string_at(map, "table"));
            let access_type = string_at(map, "access_type");

            if let Some(node_type) = node_type.as_deref() {
                facts.node_count += 1;
                if is_full_scan(node_type) {
                    let label = relation.clone().unwrap_or_else(|| node_type.to_string());
                    full_scans.insert(format!("{label} ({node_type})"));
                }
                if node_type.to_ascii_lowercase().contains("index") {
                    facts.uses_index = true;
                }
            }
            if let Some(access_type) = access_type.as_deref() {
                facts.node_count = facts.node_count.max(1);
                if access_type.eq_ignore_ascii_case("all") {
                    let label = relation.clone().unwrap_or_else(|| "table".to_string());
                    full_scans.insert(format!("{label} (access_type ALL)"));
                } else if access_type.to_ascii_lowercase().contains("index")
                    || matches!(
                        access_type.to_ascii_lowercase().as_str(),
                        "ref" | "eq_ref" | "range" | "const"
                    )
                {
                    facts.uses_index = true;
                }
            }
            if let Some(relation) = relation {
                if !relation.is_empty() {
                    relations.insert(relation);
                }
            }
            if map.contains_key("Plan Rows")
                || map.contains_key("rows")
                || map.contains_key("rows_examined_per_scan")
                || map.contains_key("rows_produced_per_join")
            {
                facts.has_row_estimates = true;
            }
            if string_at(map, "Index Name").is_some() || map.contains_key("key") {
                facts.uses_index = true;
            }
            for child in map.values() {
                walk(child, facts, relations, full_scans, depth + 1);
            }
        }
        _ => {}
    }
}

fn string_at(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn is_full_scan(node_type: &str) -> bool {
    let node = node_type.to_ascii_lowercase();
    node.contains("seq scan")
        || node.contains("table scan")
        || node.contains("full scan")
        || node.contains("scan table")
        || node == "scan"
}

/// SQLite returns plan rows as text, and plugin drivers may return a textual
/// plan. Pull the same signals out of that text.
fn scan_text(
    text: &str,
    facts: &mut PlanFacts,
    relations: &mut BTreeSet<String>,
    full_scans: &mut BTreeSet<String>,
) {
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        facts.node_count += 1;
        let upper = line.to_ascii_uppercase();
        if upper.contains("SCAN ")
            && !upper.contains("USING INDEX")
            && !upper.contains("USING COVERING INDEX")
        {
            if let Some(name) = token_after(line, "SCAN ") {
                relations.insert(name.clone());
                full_scans.insert(format!("{name} (SCAN)"));
            }
        } else if upper.contains("SEARCH ") {
            if let Some(name) = token_after(line, "SEARCH ") {
                relations.insert(name);
            }
        }
        if upper.contains("USING INDEX") || upper.contains("USING COVERING INDEX") {
            facts.uses_index = true;
        }
        if upper.contains("ESTIMATE") || upper.contains("ROWS") {
            facts.has_row_estimates = true;
        }
    }
}

fn token_after(line: &str, marker: &str) -> Option<String> {
    let upper = line.to_ascii_uppercase();
    let position = upper.find(&marker.to_ascii_uppercase())?;
    let rest = &line[position + marker.len()..];
    let token = rest
        .split(|character: char| character.is_whitespace() || character == '(')
        .next()
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();
    (!token.is_empty()).then_some(token)
}

fn build_hints(facts: &PlanFacts) -> Vec<String> {
    let mut hints = Vec::new();
    if !facts.full_scans.is_empty() {
        hints.push(format!(
            "Sequential/table scans: {}. Check whether an index on the filter or join columns would help.",
            facts.full_scans.join(", ")
        ));
    }
    if facts.node_count > 0 && !facts.uses_index {
        hints.push("No index usage was detected in this plan.".to_string());
    }
    if facts.has_row_estimates {
        hints.push(
            "The plan carries row estimates; compare them with the actual row counts for the hot step."
                .to_string(),
        );
    }
    if facts.node_count == 0 {
        hints.push(
            "The plan format was not recognized; read the raw plan in the tool result.".to_string(),
        );
    }
    hints
}
