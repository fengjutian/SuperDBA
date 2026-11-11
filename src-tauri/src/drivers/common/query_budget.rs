use serde::Serialize;

/// Maximum serialized result materialized by an automated read. The
/// application layer enforces the same ceiling again on the final envelope.
pub const AUTOMATED_MAX_RESULT_BYTES: usize = 2 * 1024 * 1024;

/// Counts JSON bytes before rows are retained in memory.
pub struct ResultByteBudget {
    limit: usize,
    used: usize,
}

impl ResultByteBudget {
    pub fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }

    /// Returns `false` without increasing the counter when the value would
    /// exceed the configured limit.
    pub fn try_record<T: Serialize>(&mut self, value: &T) -> Result<bool, String> {
        let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        let Some(next) = self.used.checked_add(bytes.len()) else {
            return Ok(false);
        };
        if next > self.limit {
            return Ok(false);
        }
        self.used = next;
        Ok(true)
    }
}
