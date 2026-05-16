/// Parsed paths from a `diff --git` file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFileHeader {
    /// The old-side path from the file header.
    pub old_path: String,
    /// The new-side path from the file header.
    pub new_path: String,
}

/// Parses a `diff --git a/path b/path` header into normalized paths.
pub fn parse_file_header(line: &str) -> Option<ParsedFileHeader> {
    let rest = line.strip_prefix("diff --git ")?;
    let (old_raw, new_raw) = split_git_paths(rest)?;
    let old_path = normalize_diff_path(&old_raw)?;
    let new_path = normalize_diff_path(&new_raw)?;

    Some(ParsedFileHeader { old_path, new_path })
}

/// Normalizes a diff path by stripping git side prefixes.
pub fn normalize_diff_path(path: &str) -> Option<String> {
    let path = unquote_path(path.trim())?;
    if path == "/dev/null" {
        return None;
    }

    Some(
        path.strip_prefix("a/")
            .or_else(|| path.strip_prefix("b/"))
            .unwrap_or(&path)
            .to_string(),
    )
}

fn split_git_paths(rest: &str) -> Option<(String, String)> {
    if rest.starts_with('"') {
        return split_quoted_git_paths(rest);
    }

    let separator = rest.find(" b/")?;
    let old_path = rest[..separator].to_string();
    let new_path = rest[separator + 1..].to_string();
    Some((old_path, new_path))
}

fn split_quoted_git_paths(rest: &str) -> Option<(String, String)> {
    let (old_path, consumed) = parse_quoted_path(rest)?;
    let next = rest[consumed..].trim_start();
    let (new_path, _) = parse_quoted_path(next)?;
    Some((old_path, new_path))
}

fn parse_quoted_path(input: &str) -> Option<(String, usize)> {
    let mut output = String::new();
    let mut escaped = false;

    for (index, character) in input.char_indices().skip(1) {
        if escaped {
            output.push(unescape_character(character));
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => return Some((output, index + 1)),
            _ => output.push(character),
        }
    }

    None
}

fn unquote_path(path: &str) -> Option<String> {
    if path.starts_with('"') {
        parse_quoted_path(path).map(|(path, _)| path)
    } else {
        Some(path.to_string())
    }
}

fn unescape_character(character: char) -> char {
    match character {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        other => other,
    }
}
