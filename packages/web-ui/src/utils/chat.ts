/** Pure helpers for the DBA chat panel. */

const SQL_FENCE = /```sql\s*([\s\S]*?)```/i;

/**
 * Returns the first fenced `sql` block in an assistant message, trimmed.
 * Returns `null` when the message has no SQL block.
 */
export function extractSqlBlock(content: string): string | null {
    const match = content.match(SQL_FENCE);
    return match ? match[1].trim() : null;
}

/**
 * Renders a tool result for the details panel without dumping megabytes into
 * the DOM. The suffix marks that the text was cut.
 */
export function formatToolResult(value: unknown, limit = 1600): string {
    if (value === undefined) return '';
    const text = JSON.stringify(value, null, 2) ?? '';
    if (text.length <= limit) return text;
    return `${text.slice(0, limit)}\n…`;
}
