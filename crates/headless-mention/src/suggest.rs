use std::ops::Range;

use crate::{MentionType, Parsed, Part};

/// The result of picking a suggestion: the new value and where the caret goes in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Insertion {
    /// The new value.
    pub value: String,
    /// The caret in the new value, in bytes, right after the mention and its space if one was added.
    pub value_caret: usize,
}

/// Where the keyword being typed for `mention_type` starts, if one is.
///
/// Returns the index of the part holding it and the byte offset of its trigger in the plain text.
pub(crate) fn find_keyword(
    parsed: &Parsed,
    mention_type: &MentionType,
    selection: Range<usize>,
) -> Option<(usize, usize)> {
    if !selection.is_empty() {
        return None;
    }

    let caret = selection.end;
    let index = parsed
        .parts
        .iter()
        .position(|part| part.range.start < caret && caret <= part.range.end)?;
    let part = &parsed.parts[index];

    if part.mention.is_some() {
        return None;
    }

    let trigger = mention_type.trigger();
    let trigger_start =
        parsed.plain_text[part.range.start..caret].rfind(trigger)? + part.range.start;

    let starts_word = parsed.plain_text[..trigger_start]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    if !starts_word {
        return None;
    }

    let typed = &parsed.plain_text[trigger_start + trigger.len()..caret];
    if typed.contains('\n') || typed.matches(' ').count() > mention_type.allowed_spaces {
        return None;
    }

    Some((index, trigger_start))
}

/// The keyword typed after the trigger of `mention_type`, if suggestions should show.
///
/// `selection` is in the plain text. There's only a keyword for a collapsed selection right after a
/// trigger that starts the text or follows whitespace, with no new line in between and no more
/// spaces than [`MentionType::allowed_spaces`].
///
/// ```
/// use headless_mention::{MentionType, keyword, parse};
///
/// let users = MentionType::new('@', |mention| format!("@user{}", mention.id));
/// let parsed = parse("hello @jo", &[users.clone().into()]);
///
/// assert_eq!(keyword(&parsed, &users, 9..9), Some("jo"));
/// assert_eq!(keyword(&parsed, &users, 3..3), None);
/// ```
pub fn keyword<'a>(
    parsed: &'a Parsed,
    mention_type: &MentionType,
    selection: Range<usize>,
) -> Option<&'a str> {
    let (_, trigger_start) = find_keyword(parsed, mention_type, selection.clone())?;

    Some(&parsed.plain_text[trigger_start + mention_type.trigger().len()..selection.end])
}

/// Replaces the keyword being typed with a mention of `id`.
///
/// Returns `None` when there's no [`keyword`] at `selection`, since there's no trigger to replace.
///
/// ```
/// use headless_mention::{MentionType, insert_mention, parse};
///
/// let users = MentionType::new('@', |mention| format!("@user{}", mention.id));
/// let parsed = parse("hello @jo", &[users.clone().into()]);
/// let insertion = insert_mention(&parsed, &users, 9..9, "42").unwrap();
///
/// assert_eq!(insertion.value, "hello <@42>");
/// assert_eq!(insertion.value_caret, insertion.value.len());
/// ```
pub fn insert_mention(
    parsed: &Parsed,
    mention_type: &MentionType,
    selection: Range<usize>,
    id: &str,
) -> Option<Insertion> {
    let (index, trigger_start) = find_keyword(parsed, mention_type, selection.clone())?;
    let caret = selection.end;
    let part = &parsed.parts[index];

    let add_space = mention_type.space_after
        && (caret == parsed.plain_text.len() || parsed.plain_text[caret..].starts_with('\n'));

    let mut value: String = parsed.parts[..index].iter().map(Part::raw).collect();
    value.push_str(&part.text[..trigger_start - part.range.start]);
    value.push_str(&mention_type.format(id));
    if add_space {
        value.push(' ');
    }

    let value_caret = value.len();

    value.push_str(&part.text[caret - part.range.start..]);
    value.extend(parsed.parts[index + 1..].iter().map(Part::raw));

    Some(Insertion { value, value_caret })
}
