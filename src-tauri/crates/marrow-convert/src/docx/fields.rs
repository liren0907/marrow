pub(super) enum FieldState {
    None,
    /// Between `fldCharType="begin"` and `"separate"` — collecting
    /// instrText.
    Begin,
    /// Between `"separate"` and `"end"` — collecting display runs.
    Separate,
}

/// Extract the URL from a HYPERLINK field-code body. Body looks like:
/// `HYPERLINK "https://example.com" \o "tooltip"` (or single-quoted, or
/// no quotes at all). Returns the URL portion when the body starts with
/// HYPERLINK; `None` otherwise (so non-link fields fall back to display
/// runs).
pub(super) fn parse_hyperlink_instr(instr: &str) -> Option<String> {
    let trimmed = instr.trim();
    let rest = trimmed.strip_prefix("HYPERLINK")?.trim_start();
    // Quoted URL.
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    // Unquoted URL up to first whitespace.
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    if end == 0 {
        None
    } else {
        Some(rest[..end].to_string())
    }
}
