//! Conservative SQL safeguards for statements proposed by AI tools.
//!
//! These checks intentionally apply only to automated DBA chat and MCP
//! execution. The SQL editor remains available for deliberate administrative
//! work that requires broader database privileges.

use crate::ai_activity;

#[cfg(test)]
mod tests;

const UNBOUNDED_WRITE_MESSAGE: &str =
    "AI query blocked: UPDATE and DELETE statements require a WHERE clause. Review and run the statement explicitly in the SQL editor if a full-table change is intended.";
const LOCK_HEAVY_MESSAGE: &str =
    "AI query blocked: lock-heavy DDL or maintenance statements must be reviewed and run explicitly in the SQL editor.";
const TRANSACTION_CONTROL_MESSAGE: &str =
    "AI query blocked: transaction-control statements are not allowed through automated channels because they can leave pooled connections holding locks. Run the transaction explicitly in the SQL editor.";
const LOCKING_READ_MESSAGE: &str =
    "AI query blocked: locking reads and advisory-lock functions are not allowed through automated channels. Run the statement explicitly in the SQL editor.";
const SQL_TOO_LARGE_MESSAGE: &str =
    "AI query blocked: the SQL text exceeds the 256 KiB automated-query limit. Review and run it explicitly in the SQL editor.";
const RESOURCE_ABUSE_MESSAGE: &str =
    "AI query blocked: delay and benchmark operations are not allowed through automated channels because they occupy database workers without useful progress.";
const AUDIT_TIME_RANGE_MESSAGE: &str =
    "AI query blocked: audit_log queries require a created_at range condition in WHERE. Start with a narrow time window and expand it explicitly if needed.";
const AUDIT_TIME_WINDOW_MESSAGE: &str =
    "AI query blocked: audit_log automated queries are limited to a 30-day time window. Split larger investigations into smaller ranges.";
const ROW_GENERATOR_MESSAGE: &str =
    "AI query blocked: recursive queries and automatic row generators must be reviewed and run explicitly in the SQL editor.";
const QUERY_COMPLEXITY_MESSAGE: &str =
    "AI query blocked: the statement exceeds automated-query complexity limits. Reduce joins, set operations, or nesting before retrying.";
const EXPENSIVE_PLAN_MESSAGE: &str =
    "AI query blocked: Cartesian joins and full-table random ordering must be reviewed and run explicitly in the SQL editor.";
const MAX_AI_SQL_BYTES: usize = 256 * 1024;
const MAX_AI_JOINS: usize = 8;
const MAX_AI_SET_OPERATIONS: usize = 8;
const MAX_AI_NESTING_DEPTH: usize = 32;
const MAX_AUDIT_WINDOW_SECONDS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    OpenParen,
    CloseParen,
    Semicolon,
    RangeComparator,
    Equal,
}

/// Reject SQL patterns that are too risky for an automated execution channel.
pub fn validate_ai_sql(sql: &str) -> Result<(), &'static str> {
    if sql.len() > MAX_AI_SQL_BYTES {
        return Err(SQL_TOO_LARGE_MESSAGE);
    }

    let tokens = tokens(sql);

    if contains_lock_heavy_operation(&tokens) {
        return Err(LOCK_HEAVY_MESSAGE);
    }
    if contains_transaction_control(&tokens) {
        return Err(TRANSACTION_CONTROL_MESSAGE);
    }
    if contains_locking_read(&tokens) {
        return Err(LOCKING_READ_MESSAGE);
    }
    if contains_resource_abuse(&tokens) {
        return Err(RESOURCE_ABUSE_MESSAGE);
    }
    if audit_log_without_time_range(&tokens) {
        return Err(AUDIT_TIME_RANGE_MESSAGE);
    }
    if audit_log_interval_too_wide(&tokens) {
        return Err(AUDIT_TIME_WINDOW_MESSAGE);
    }
    if contains_unbounded_row_generator(&tokens) {
        return Err(ROW_GENERATOR_MESSAGE);
    }
    if exceeds_query_complexity(&tokens) {
        return Err(QUERY_COMPLEXITY_MESSAGE);
    }
    if contains_expensive_plan_pattern(&tokens) {
        return Err(EXPENSIVE_PLAN_MESSAGE);
    }
    if contains_unbounded_write(&tokens) {
        return Err(UNBOUNDED_WRITE_MESSAGE);
    }

    Ok(())
}

fn tokens(sql: &str) -> Vec<Token> {
    let stripped = ai_activity::strip_strings_and_comments(sql);
    let mut result = Vec::new();
    let mut word = String::new();

    let flush_word = |word: &mut String, result: &mut Vec<Token>| {
        if !word.is_empty() {
            result.push(Token::Word(std::mem::take(word).to_ascii_uppercase()));
        }
    };

    for character in stripped.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            word.push(character);
            continue;
        }

        flush_word(&mut word, &mut result);
        match character {
            '(' => result.push(Token::OpenParen),
            ')' => result.push(Token::CloseParen),
            ';' => result.push(Token::Semicolon),
            '<' | '>' => result.push(Token::RangeComparator),
            '=' => result.push(Token::Equal),
            _ => {}
        }
    }
    flush_word(&mut word, &mut result);
    result
}

fn contains_lock_heavy_operation(tokens: &[Token]) -> bool {
    top_level_statement_prefixes(tokens).any(|words| match words.as_slice() {
        ["ALTER", ..]
        | ["TRUNCATE", ..]
        | ["VACUUM", ..]
        | ["OPTIMIZE", ..]
        | ["REINDEX", ..]
        | ["CLUSTER", ..]
        | ["LOCK", ..]
        | ["CREATE", "INDEX", ..]
        | ["CREATE", "UNIQUE", "INDEX", ..]
        | ["DROP", "INDEX", ..] => true,
        _ => false,
    })
}

fn contains_transaction_control(tokens: &[Token]) -> bool {
    top_level_statement_prefixes(tokens).any(|words| match words.as_slice() {
        ["BEGIN", ..]
        | ["START", "TRANSACTION", ..]
        | ["COMMIT", ..]
        | ["END", ..]
        | ["ROLLBACK", ..]
        | ["SAVEPOINT", ..]
        | ["RELEASE", "SAVEPOINT", ..]
        | ["SET", "TRANSACTION", ..]
        | ["SET", "SESSION", "TRANSACTION", ..]
        | ["SET", "AUTOCOMMIT", ..] => true,
        _ => false,
    })
}

fn contains_locking_read(tokens: &[Token]) -> bool {
    let words = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Word(word) => Some(word.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    words.iter().any(|word| {
        matches!(
            *word,
            "PG_ADVISORY_LOCK"
                | "PG_ADVISORY_XACT_LOCK"
                | "PG_TRY_ADVISORY_LOCK"
                | "PG_TRY_ADVISORY_XACT_LOCK"
                | "GET_LOCK"
                | "SP_GETAPPLOCK"
        )
    }) || words
        .windows(2)
        .any(|pair| matches!(pair, ["FOR", "UPDATE" | "SHARE"]))
        || words
            .windows(3)
            .any(|triple| matches!(triple, ["FOR", "KEY" | "NO", "SHARE" | "KEY"]))
        || words.windows(4).any(|quad| {
            matches!(
                quad,
                ["FOR", "NO", "KEY", "UPDATE"] | ["LOCK", "IN", "SHARE", "MODE"]
            )
        })
}

fn contains_resource_abuse(tokens: &[Token]) -> bool {
    const BLOCKED_FUNCTIONS: &[&str] = &[
        "PG_SLEEP",
        "PG_SLEEP_FOR",
        "PG_SLEEP_UNTIL",
        "SLEEP",
        "BENCHMARK",
    ];

    tokens.windows(2).any(|pair| match pair {
        [Token::Word(word), Token::OpenParen] => BLOCKED_FUNCTIONS.contains(&word.as_str()),
        _ => false,
    }) || top_level_statement_prefixes(tokens)
        .any(|words| matches!(words.as_slice(), ["WAITFOR", "DELAY" | "TIME", ..]))
}

fn audit_log_without_time_range(tokens: &[Token]) -> bool {
    let references_audit_log = tokens
        .iter()
        .any(|token| matches!(token, Token::Word(word) if word == "AUDIT_LOG"));
    if !references_audit_log {
        return false;
    }

    tokens.iter().enumerate().all(|(index, token)| {
        if !matches!(token, Token::Word(word) if word == "WHERE") {
            return true;
        }

        !has_created_at_range(&tokens[index + 1..])
    })
}

fn has_created_at_range(tokens: &[Token]) -> bool {
    tokens.windows(2).any(|pair| {
        matches!(
            pair,
            [Token::Word(column), Token::RangeComparator]
                if column == "CREATED_AT"
        ) || matches!(
            pair,
            [Token::RangeComparator, Token::Word(column)]
                if column == "CREATED_AT"
        ) || matches!(
            pair,
            [Token::Word(column), Token::Word(operator)]
                if column == "CREATED_AT" && operator == "BETWEEN"
        )
    })
}

fn audit_log_interval_too_wide(tokens: &[Token]) -> bool {
    let references_audit_log = tokens
        .iter()
        .any(|token| matches!(token, Token::Word(word) if word == "AUDIT_LOG"));
    if !references_audit_log {
        return false;
    }

    tokens.windows(3).any(|window| match window {
        [Token::Word(interval), Token::Word(amount), Token::Word(unit)]
            if interval == "INTERVAL" =>
        {
            let Ok(amount) = amount.parse::<u64>() else {
                return false;
            };
            let seconds_per_unit = match unit.as_str() {
                "SECOND" | "SECONDS" => 1,
                "MINUTE" | "MINUTES" => 60,
                "HOUR" | "HOURS" => 60 * 60,
                "DAY" | "DAYS" => 24 * 60 * 60,
                "WEEK" | "WEEKS" => 7 * 24 * 60 * 60,
                "MONTH" | "MONTHS" | "YEAR" | "YEARS" => return true,
                _ => return false,
            };
            amount.saturating_mul(seconds_per_unit) > MAX_AUDIT_WINDOW_SECONDS
        }
        _ => false,
    })
}

fn top_level_statement_prefixes(tokens: &[Token]) -> impl Iterator<Item = Vec<&str>> {
    let mut prefixes = Vec::new();
    let mut prefix = Vec::new();
    let mut depth = 0usize;

    for token in tokens {
        match token {
            Token::OpenParen => depth += 1,
            Token::CloseParen => depth = depth.saturating_sub(1),
            Token::Semicolon if depth == 0 => {
                if !prefix.is_empty() {
                    prefixes.push(std::mem::take(&mut prefix));
                }
            }
            Token::Word(word) if depth == 0 && prefix.len() < 3 => prefix.push(word.as_str()),
            _ => {}
        }
    }
    if !prefix.is_empty() {
        prefixes.push(prefix);
    }

    prefixes.into_iter()
}

fn contains_unbounded_row_generator(tokens: &[Token]) -> bool {
    tokens.windows(2).any(|pair| {
        matches!(
            pair,
            [Token::Word(with), Token::Word(recursive)]
                if with == "WITH" && recursive == "RECURSIVE"
        ) || matches!(
            pair,
            [Token::Word(function), Token::OpenParen]
                if function == "GENERATE_SERIES"
        )
    })
}

fn exceeds_query_complexity(tokens: &[Token]) -> bool {
    let mut joins = 0usize;
    let mut set_operations = 0usize;
    let mut depth = 0usize;

    for token in tokens {
        match token {
            Token::OpenParen => {
                depth += 1;
                if depth > MAX_AI_NESTING_DEPTH {
                    return true;
                }
            }
            Token::CloseParen => depth = depth.saturating_sub(1),
            Token::Word(word) if word == "JOIN" => {
                joins += 1;
                if joins > MAX_AI_JOINS {
                    return true;
                }
            }
            Token::Word(word) if word == "UNION" => {
                set_operations += 1;
                if set_operations > MAX_AI_SET_OPERATIONS {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn contains_expensive_plan_pattern(tokens: &[Token]) -> bool {
    let has_cross_join = tokens.windows(2).any(|pair| {
        matches!(
            pair,
            [Token::Word(cross), Token::Word(join)] if cross == "CROSS" && join == "JOIN"
        )
    });
    let has_random_sort = tokens.windows(4).any(|window| {
        matches!(
            window,
            [
                Token::Word(order),
                Token::Word(by),
                Token::Word(function),
                Token::OpenParen
            ] if order == "ORDER"
                && by == "BY"
                && matches!(function.as_str(), "RAND" | "RANDOM" | "NEWID" | "UUID")
        )
    });

    has_cross_join || has_random_sort
}

fn contains_unbounded_write(tokens: &[Token]) -> bool {
    let mut depth = 0usize;

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::OpenParen => depth += 1,
            Token::CloseParen => depth = depth.saturating_sub(1),
            Token::Word(word) if word == "UPDATE" || word == "DELETE" => {
                if !has_where_in_scope(&tokens[index + 1..], depth) {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn has_where_in_scope(tokens: &[Token], statement_depth: usize) -> bool {
    let mut depth = statement_depth;

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::OpenParen => depth += 1,
            Token::CloseParen if depth <= statement_depth => return false,
            Token::CloseParen => depth -= 1,
            Token::Semicolon if depth == statement_depth => return false,
            Token::Word(word) if depth == statement_depth && word == "WHERE" => {
                return !where_clause_is_obviously_tautological(
                    &tokens[index + 1..],
                    statement_depth,
                );
            }
            _ => {}
        }
    }

    false
}

fn where_clause_is_obviously_tautological(tokens: &[Token], statement_depth: usize) -> bool {
    let mut depth = statement_depth;
    let mut condition = Vec::new();

    for token in tokens {
        match token {
            Token::OpenParen => {
                depth += 1;
                condition.push(token);
            }
            Token::CloseParen if depth <= statement_depth => break,
            Token::CloseParen => {
                depth -= 1;
                condition.push(token);
            }
            Token::Semicolon if depth == statement_depth => break,
            Token::Word(word)
                if depth == statement_depth
                    && matches!(word.as_str(), "RETURNING" | "ORDER" | "GROUP" | "LIMIT") =>
            {
                break;
            }
            _ => condition.push(token),
        }
    }

    let normalized = condition
        .into_iter()
        .filter(|token| !matches!(token, Token::OpenParen | Token::CloseParen))
        .collect::<Vec<_>>();

    is_tautology(&normalized)
        || normalized.iter().enumerate().any(|(index, token)| {
            matches!(token, Token::Word(word) if word == "OR")
                && is_tautology(&normalized[index + 1..])
        })
}

fn is_tautology(tokens: &[&Token]) -> bool {
    matches!(tokens, [Token::Word(value)] if value == "TRUE")
        || matches!(tokens, [Token::Word(not), Token::Word(value)] if not == "NOT" && value == "FALSE")
        || matches!(
            tokens,
            [Token::Word(left), Token::Equal, Token::Word(right)] if left == right
        )
}
