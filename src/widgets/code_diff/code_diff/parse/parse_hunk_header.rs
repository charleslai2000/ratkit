/// Parsed source ranges from a unified diff hunk header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedHunkHeader {
    /// First old-file line covered by the hunk.
    pub old_start: usize,
    /// Number of old-file lines covered by the hunk.
    pub old_count: usize,
    /// First new-file line covered by the hunk.
    pub new_start: usize,
    /// Number of new-file lines covered by the hunk.
    pub new_count: usize,
}

/// Parses a unified diff hunk header into old and new source ranges.
pub fn parse_hunk_header(header: &str) -> Option<ParsedHunkHeader> {
    let header = header.strip_prefix("@@")?;
    let range_text = header.split("@@").next()?.trim();
    let mut ranges = range_text.split_whitespace();
    let old_range = ranges.next()?.strip_prefix('-')?;
    let new_range = ranges.next()?.strip_prefix('+')?;
    let (old_start, old_count) = parse_range(old_range)?;
    let (new_start, new_count) = parse_range(new_range)?;

    Some(ParsedHunkHeader {
        old_start,
        old_count,
        new_start,
        new_count,
    })
}

fn parse_range(range: &str) -> Option<(usize, usize)> {
    let mut parts = range.split(',');
    let start = parts.next()?.parse().ok()?;
    let count = parts.next().map(str::parse).transpose().ok()?.unwrap_or(1);
    Some((start, count))
}
