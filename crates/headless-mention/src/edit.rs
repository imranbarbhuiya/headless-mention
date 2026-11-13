use std::ops::Range;

use crate::{
    Parsed, Part,
    diff::{Change, changes},
};

/// The new value after the user changed the plain text of `parsed` to `changed`.
///
/// Mentions whose labels are untouched stay mentions. A mention whose label was edited, even
/// partly, turns into its remaining label as plain text.
///
/// ```
/// use headless_mention::{MentionType, PartType, apply_text_change, parse};
///
/// let types = [PartType::from(MentionType::new('@', |mention| format!("@user{}", mention.id)))];
/// let parsed = parse("hi <@1>", &types);
///
/// assert_eq!(apply_text_change(&parsed, "hey hi @user1!"), "hey hi <@1>!");
/// assert_eq!(apply_text_change(&parsed, "hi @use"), "hi @use");
/// ```
pub fn apply_text_change(parsed: &Parsed, changed: &str) -> String {
    let mut value = String::with_capacity(changed.len());
    let mut cursor = 0;

    for change in changes(&parsed.plain_text, changed) {
        match change {
            Change::Keep(len) => {
                push_interval(&mut value, &parsed.parts, cursor..cursor + len);
                cursor += len;
            }
            Change::Remove(len) => cursor += len,
            Change::Add(text) => value.push_str(text),
        }
    }

    value
}

/// Appends the value of the plain text in `range`.
///
/// A part covered whole keeps its raw text, so a mention stays a mention. A part covered only
/// partly contributes the covered slice of its shown text, so a cut mention becomes plain text.
fn push_interval(value: &mut String, parts: &[Part], range: Range<usize>) {
    let Some(first) = parts.iter().position(|part| part.range.end > range.start) else {
        return;
    };
    let Some(last) = parts.iter().position(|part| part.range.end >= range.end) else {
        return;
    };

    for part in &parts[first..=last] {
        let start = range.start.max(part.range.start);
        let end = range.end.min(part.range.end);

        if start == part.range.start && end == part.range.end {
            value.push_str(part.raw());
        } else {
            value.push_str(&part.text[start - part.range.start..end - part.range.start]);
        }
    }
}
