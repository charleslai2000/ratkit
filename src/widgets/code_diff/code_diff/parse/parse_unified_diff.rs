use super::{normalize_diff_path, parse_file_header, parse_hunk_header};
use crate::widgets::code_diff::code_diff::types::{DiffFile, DiffFileStatus, DiffHunk, DiffLine};

/// Parses a unified diff string into file entries with hunks and line numbers.
pub fn parse_unified_diff(diff: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut pending_old_path: Option<String> = None;
    let mut old_line = 0;
    let mut new_line = 0;

    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            finish_file(&mut files, &mut current_file, &mut current_hunk);
            pending_old_path = None;
            current_file = parse_file_header(line).map(|header| {
                let mut file = DiffFile::new(header.new_path);
                file.old_path = Some(header.old_path);
                file
            });
            continue;
        }

        if current_file.is_none()
            && apply_pending_file_header(line, &mut pending_old_path, &mut current_file)
        {
            continue;
        }

        if current_file.is_none() && line.starts_with("@@") {
            current_file = Some(DiffFile::new("(diff)"));
        }

        let Some(file) = current_file.as_mut() else {
            continue;
        };

        if apply_file_metadata(file, line) {
            continue;
        }

        if line.starts_with("@@") {
            finish_hunk(file, &mut current_hunk);
            if let Some(header) = parse_hunk_header(line) {
                let mut hunk = DiffHunk::new(
                    header.old_start,
                    header.old_count,
                    header.new_start,
                    header.new_count,
                );
                hunk.header = line.to_string();
                old_line = header.old_start;
                new_line = header.new_start;
                current_hunk = Some(hunk);
            }
            continue;
        }

        if let Some(hunk) = current_hunk.as_mut() {
            apply_hunk_line(hunk, line, &mut old_line, &mut new_line);
        }
    }

    finish_file(&mut files, &mut current_file, &mut current_hunk);
    files
}

fn apply_pending_file_header(
    line: &str,
    pending_old_path: &mut Option<String>,
    current_file: &mut Option<DiffFile>,
) -> bool {
    if let Some(path) = line.strip_prefix("--- ") {
        *pending_old_path = normalize_diff_path(path);
        return true;
    }

    if let Some(path) = line.strip_prefix("+++ ") {
        let new_path = normalize_diff_path(path);
        let display_path = new_path.clone().or_else(|| pending_old_path.clone());
        if let Some(display_path) = display_path {
            let mut file = DiffFile::new(display_path);
            file.old_path = pending_old_path.clone();
            match (pending_old_path.is_some(), new_path.is_some()) {
                (false, true) => file.status = DiffFileStatus::Added,
                (true, false) => file.status = DiffFileStatus::Deleted,
                _ => {}
            }
            *current_file = Some(file);
        }
        return true;
    }

    false
}

fn apply_file_metadata(file: &mut DiffFile, line: &str) -> bool {
    if line.starts_with("new file mode ") {
        file.status = DiffFileStatus::Added;
        return true;
    }
    if line.starts_with("deleted file mode ") {
        file.status = DiffFileStatus::Deleted;
        return true;
    }
    if let Some(path) = line.strip_prefix("rename from ") {
        file.old_path = Some(path.to_string());
        file.status = DiffFileStatus::Renamed;
        return true;
    }
    if let Some(path) = line.strip_prefix("rename to ") {
        file.path = path.to_string();
        file.status = DiffFileStatus::Renamed;
        return true;
    }
    if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
        file.is_binary = true;
        file.status = DiffFileStatus::Binary;
        return true;
    }
    if let Some(path) = line.strip_prefix("--- ") {
        apply_old_path(file, path);
        return true;
    }
    if let Some(path) = line.strip_prefix("+++ ") {
        apply_new_path(file, path);
        return true;
    }
    false
}

fn apply_hunk_line(hunk: &mut DiffHunk, line: &str, old_line: &mut usize, new_line: &mut usize) {
    match line.chars().next() {
        Some(' ') => {
            hunk.add_line(DiffLine::context(&line[1..], *old_line, *new_line));
            *old_line += 1;
            *new_line += 1;
        }
        Some('-') if !line.starts_with("---") => {
            hunk.add_line(DiffLine::removed(&line[1..], *old_line));
            *old_line += 1;
        }
        Some('+') if !line.starts_with("+++") => {
            hunk.add_line(DiffLine::added(&line[1..], *new_line));
            *new_line += 1;
        }
        Some('\t') => {
            hunk.add_line(DiffLine::context(line, *old_line, *new_line));
            *old_line += 1;
            *new_line += 1;
        }
        _ => {}
    }
}

fn apply_old_path(file: &mut DiffFile, path: &str) {
    match normalize_diff_path(path) {
        Some(path) => file.old_path = Some(path),
        None => {
            file.old_path = None;
            file.status = DiffFileStatus::Added;
        }
    }
}

fn apply_new_path(file: &mut DiffFile, path: &str) {
    match normalize_diff_path(path) {
        Some(path) => file.path = path,
        None => file.status = DiffFileStatus::Deleted,
    }
}

fn finish_file(
    files: &mut Vec<DiffFile>,
    current_file: &mut Option<DiffFile>,
    current_hunk: &mut Option<DiffHunk>,
) {
    if let Some(file) = current_file.as_mut() {
        finish_hunk(file, current_hunk);
    }
    if let Some(file) = current_file.take() {
        files.push(file);
    }
}

fn finish_hunk(file: &mut DiffFile, current_hunk: &mut Option<DiffHunk>) {
    if let Some(hunk) = current_hunk.take() {
        file.add_hunk(hunk);
    }
}
