use crate::drivers::driver_trait::SqlDialect;
use sqlparser::ast::visit_relations;
use sqlparser::dialect::{
    GenericDialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect,
};
use sqlparser::parser::Parser;
use std::collections::BTreeSet;
use std::ops::ControlFlow;

pub fn extract_sql_resources(sql: &str, dialect: SqlDialect) -> Result<Vec<String>, String> {
    let statements = match dialect {
        SqlDialect::Postgres => Parser::parse_sql(&PostgreSqlDialect {}, sql),
        SqlDialect::Mysql => Parser::parse_sql(&MySqlDialect {}, sql),
        SqlDialect::Mssql => Parser::parse_sql(&MsSqlDialect {}, sql),
        SqlDialect::Sqlite => Parser::parse_sql(&SQLiteDialect {}, sql),
        SqlDialect::Oracle | SqlDialect::Generic => Parser::parse_sql(&GenericDialect {}, sql),
    }
    .map_err(|error| format!("Security policy could not parse SQL resources: {error}"))?;

    let mut resources = BTreeSet::new();
    visit_relations(&statements, |relation| {
        let normalized = relation
            .0
            .iter()
            .map(|part| part.value.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(".");
        if !normalized.is_empty() {
            resources.insert(normalized);
        }
        ControlFlow::<()>::Continue(())
    });
    Ok(resources.into_iter().collect())
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
}
