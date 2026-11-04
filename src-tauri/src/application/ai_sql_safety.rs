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
const MAX_AI_SQL_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    OpenParen,
    CloseParen,
    Semicolon,
    RangeComparator,
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

    for token in tokens {
        match token {
            Token::OpenParen => depth += 1,
            Token::CloseParen if depth <= statement_depth => return false,
            Token::CloseParen => depth -= 1,
            Token::Semicolon if depth == statement_depth => return false,
            Token::Word(word) if depth == statement_depth && word == "WHERE" => return true,
            _ => {}
        }
    }

    false
}
