use std::ops::Range;

use similar::{Algorithm, DiffOp, capture_diff_slices};

/// One step of turning the old plain text into the new one.
pub(crate) enum Change<'a> {
    /// This many bytes of the old plain text are kept.
    Keep(usize),
    /// This many bytes of the old plain text are removed.
    Remove(usize),
    /// This text is added.
    Add(&'a str),
}

/// A keystroke, backspace or paste only adds or removes one run, which the common prefix and
/// suffix already describe. Only a replacement needs a character diff of the middle, which is what
/// keeps a mention inside the replaced range intact.
pub(crate) fn changes<'a>(old: &str, new: &'a str) -> Vec<Change<'a>> {
    let prefix = common_prefix(old, new);
    let suffix = common_suffix(&old[prefix..], &new[prefix..]);

    let removed = &old[prefix..old.len() - suffix];
    let added = &new[prefix..new.len() - suffix];

    let mut changes = Vec::new();

    if prefix > 0 {
        changes.push(Change::Keep(prefix));
    }

    if !removed.is_empty() && !added.is_empty() {
        diff_chars(removed, added, &mut changes);
    } else {
        if !removed.is_empty() {
            changes.push(Change::Remove(removed.len()));
        }
        if !added.is_empty() {
            changes.push(Change::Add(added));
        }
    }

    if suffix > 0 {
        changes.push(Change::Keep(suffix));
    }

    changes
}

fn diff_chars<'a>(old: &str, new: &'a str, changes: &mut Vec<Change<'a>>) {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let old_offsets = char_offsets(old);
    let new_offsets = char_offsets(new);

    let old_len = |range: Range<usize>| old_offsets[range.end] - old_offsets[range.start];
    let new_text = |range: Range<usize>| &new[new_offsets[range.start]..new_offsets[range.end]];

    for op in capture_diff_slices(Algorithm::Myers, &old_chars, &new_chars) {
        match op {
            DiffOp::Equal { .. } => changes.push(Change::Keep(old_len(op.old_range()))),
            DiffOp::Delete { .. } => changes.push(Change::Remove(old_len(op.old_range()))),
            DiffOp::Insert { .. } => changes.push(Change::Add(new_text(op.new_range()))),
            DiffOp::Replace { .. } => {
                changes.push(Change::Remove(old_len(op.old_range())));
                changes.push(Change::Add(new_text(op.new_range())));
            }
        }
    }
}

/// The byte offset of every char, followed by the length.
fn char_offsets(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(offset, _)| offset)
        .chain([text.len()])
        .collect()
}

fn common_prefix(a: &str, b: &str) -> usize {
    a.char_indices()
        .zip(b.chars())
        .find(|((_, x), y)| x != y)
        .map_or(a.len().min(b.len()), |((offset, _), _)| offset)
}

fn common_suffix(a: &str, b: &str) -> usize {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum()
}
