use crate::drivers::driver_trait::SqlDialect;
use sqlparser::ast::{visit_relations, Expr, Query, SelectItem, SetExpr, Visit, Visitor};
use sqlparser::dialect::{
    GenericDialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect,
};
use sqlparser::parser::Parser;
use std::collections::BTreeSet;
use std::ops::ControlFlow;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SqlAccess {
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub column_wildcard: bool,
}

#[derive(Default)]
struct AccessCollector {
    aliases: BTreeSet<String>,
    columns: BTreeSet<String>,
    column_wildcard: bool,
}

impl Visitor for AccessCollector {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        if let Some(with) = &query.with {
            self.aliases.extend(
                with.cte_tables
                    .iter()
                    .map(|cte| cte.alias.name.value.to_ascii_lowercase()),
            );
        }
        if let SetExpr::Select(select) = query.body.as_ref() {
            self.column_wildcard |= select.projection.iter().any(|item| {
                matches!(
                    item,
                    SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _)
                )
            });
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        let column = match expr {
            Expr::Identifier(identifier) => Some(identifier.value.to_ascii_lowercase()),
            Expr::CompoundIdentifier(parts) => Some(
                parts
                    .iter()
                    .map(|part| part.value.to_ascii_lowercase())
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            _ => None,
        };
        if let Some(column) = column.filter(|column| !column.is_empty()) {
            self.columns.insert(column);
        }
        ControlFlow::Continue(())
    }
}

pub fn extract_sql_access(sql: &str, dialect: SqlDialect) -> Result<SqlAccess, String> {
    let statements = match dialect {
        SqlDialect::Postgres => Parser::parse_sql(&PostgreSqlDialect {}, sql),
        SqlDialect::Mysql => Parser::parse_sql(&MySqlDialect {}, sql),
        SqlDialect::Mssql => Parser::parse_sql(&MsSqlDialect {}, sql),
        SqlDialect::Sqlite => Parser::parse_sql(&SQLiteDialect {}, sql),
        SqlDialect::Oracle | SqlDialect::Generic => Parser::parse_sql(&GenericDialect {}, sql),
    }
    .map_err(|error| format!("Security policy could not parse SQL resources: {error}"))?;

    let mut access = AccessCollector::default();
    let _ = statements.visit(&mut access);
    let mut tables = BTreeSet::new();
    let _ = visit_relations(&statements, |relation| {
        let normalized = relation
            .0
            .iter()
            .map(|part| part.value.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(".");
        if !normalized.is_empty() && !access.aliases.contains(&normalized) {
            tables.insert(normalized);
        }
        ControlFlow::<()>::Continue(())
    });
    Ok(SqlAccess {
        tables: tables.into_iter().collect(),
        columns: access.columns.into_iter().collect(),
        column_wildcard: access.column_wildcard,
    })
}

pub fn extract_sql_resources(sql: &str, dialect: SqlDialect) -> Result<Vec<String>, String> {
    extract_sql_access(sql, dialect).map(|access| access.tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_qualified_joins_and_nested_queries() {
        let resources = extract_sql_resources(
            "SELECT * FROM app.users u JOIN audit.events e ON e.user_id = u.id WHERE EXISTS (SELECT 1 FROM flags)",
            SqlDialect::Postgres,
        )
        .unwrap();
        assert_eq!(resources, vec!["app.users", "audit.events", "flags"]);
    }

    #[test]
    fn extracts_write_targets_and_sources() {
        let resources = extract_sql_resources(
            "INSERT INTO archive_users SELECT * FROM users",
            SqlDialect::Postgres,
        )
        .unwrap();
        assert_eq!(resources, vec!["archive_users", "users"]);
    }

    #[test]
    fn does_not_report_cte_aliases_as_tables() {
        let resources = extract_sql_resources(
            "WITH recent AS (SELECT * FROM orders) SELECT * FROM recent",
            SqlDialect::Postgres,
        )
        .unwrap();
        assert_eq!(resources, vec!["orders"]);
    }

    #[test]
    fn respects_plugin_declared_dialects() {
        assert_eq!(
            extract_sql_resources("SELECT * FROM `sales`.`orders`", SqlDialect::Mysql).unwrap(),
            vec!["sales.orders"]
        );
        assert_eq!(
            extract_sql_resources("SELECT * FROM [sales].[orders]", SqlDialect::Mssql).unwrap(),
            vec!["sales.orders"]
        );
    }

    #[test]
    fn rejects_unparseable_sql() {
        assert!(extract_sql_resources("SELECT * FROM", SqlDialect::Generic).is_err());
    }

    #[test]
    fn extracts_columns_from_projection_filters_and_joins() {
        let access = extract_sql_access(
            "SELECT u.id, o.total FROM users u JOIN orders o ON o.user_id = u.id WHERE u.email = 'a@example.com'",
            SqlDialect::Postgres,
        )
        .unwrap();
        assert_eq!(
            access.columns,
            vec!["o.total", "o.user_id", "u.email", "u.id"]
        );
        assert!(!access.column_wildcard);
    }

    #[test]
    fn marks_qualified_and_unqualified_projection_wildcards() {
        let all = extract_sql_access("SELECT * FROM users", SqlDialect::Postgres).unwrap();
        assert!(all.column_wildcard);
        let qualified =
            extract_sql_access("SELECT users.* FROM users", SqlDialect::Postgres).unwrap();
        assert!(qualified.column_wildcard);
    }
}
