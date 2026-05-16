use similar::{ChangeTag, TextDiff};

use crate::widgets::code_diff::code_diff::types::InlineSegment;

const MIN_INLINE_SIMILARITY: f32 = 0.35;

/// Computes word-level inline emphasis for similar replacement lines.
pub fn compute_inline_segments(old: &str, new: &str) -> (Vec<InlineSegment>, Vec<InlineSegment>) {
    let diff = TextDiff::configure().diff_unicode_words(old, new);
    let unchanged = diff
        .iter_all_changes()
        .filter(|change| change.tag() == ChangeTag::Equal)
        .map(|change| change.value().chars().count())
        .sum::<usize>();

    if old == new || unchanged_ratio(unchanged, old, new) < MIN_INLINE_SIMILARITY {
        return (
            vec![InlineSegment::new(old, false)],
            vec![InlineSegment::new(new, false)],
        );
    }

    let mut old_segments = Vec::new();
    let mut new_segments = Vec::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                push_segment(&mut old_segments, change.value(), false);
                push_segment(&mut new_segments, change.value(), false);
            }
            ChangeTag::Delete => push_segment(&mut old_segments, change.value(), true),
            ChangeTag::Insert => push_segment(&mut new_segments, change.value(), true),
        }
    }

    (old_segments, new_segments)
}

fn unchanged_ratio(unchanged: usize, old: &str, new: &str) -> f32 {
    let total = old.chars().count().max(new.chars().count());

    if total == 0 {
        1.0
    } else {
        unchanged as f32 / total as f32
    }
}

fn push_segment(segments: &mut Vec<InlineSegment>, text: &str, emphasized: bool) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = segments.last_mut() {
        if last.emphasized == emphasized {
            last.text.push_str(text);
            return;
        }
    }

    segments.push(InlineSegment::new(text, emphasized));
}
